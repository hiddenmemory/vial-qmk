// This is a modification of this: https://github.com/not-nullptr/qmk-rs/tree/master/rust

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use image::{ImageBuffer, Rgba};
use proc_macro::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{Token, parse::Parse, parse_macro_input};

mod palette256;

fn remove_non_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
    re.replace_all(input, "").to_string()
}

fn to_rgb565a_with_alpha(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: usize,
    height: usize,
    has_alpha: bool,
) -> Vec<u8> {
    let mut pixel_output = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];

            pixel_output.push((red & 0xF8) | (green >> 5));
            pixel_output.push((green & 0b11111100) << 3 | (blue >> 3));

            if has_alpha {
                let alpha = pixel[3];
                pixel_output.push(alpha);
            }
        }
    }

    pixel_output
}

#[inline]
fn rgb_to_index_key(r: u8, g: u8, b: u8) -> usize {
    ((r as usize) * 100_000) + ((g as usize) * 1000) + b as usize
}

fn to_rgba_palette256_with_alpha(
    path: &str,
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: usize,
    height: usize,
    has_alpha: bool,
    palette_size: u32,
) -> Vec<u8> {
    println!("[include_image] {path} → generating median cut palette with depth {palette_size}");
    let pixels: Vec<[u8; 3]> = img.pixels().map(|p| [p[0], p[1], p[2]]).collect();
    let palette = palette256::median_cut(&pixels, palette_size); // 2^palette_size
    let mut pixel_output = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];

            let palette_id = palette256::closest_palette_index(&palette, red, green, blue);

            pixel_output.push(palette_id);

            if has_alpha {
                let alpha = pixel[3];
                pixel_output.push(alpha);
            }
        }
    }

    let mut output = Vec::with_capacity(1 + palette.len() + pixel_output.len());
    output.push(palette.len() as u8);

    palette.iter().for_each(|(r, g, b)| {
        output.push(*r);
        output.push(*g);
        output.push(*b);
    });

    output.extend_from_slice(&pixel_output);
    output
}

fn to_rgba_palette_with_alpha(
    path: &str,
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: usize,
    height: usize,
    has_alpha: bool,
) -> Option<Vec<u8>> {
    let mut palette_store: HashMap<usize, u8> = HashMap::new();
    let mut palette_output: Vec<u8> = Vec::new();
    let mut pixel_output = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];

            let key = rgb_to_index_key(red, green, blue);
            let palette_id = if let Some(id) = palette_store.get(&key) {
                *id
            } else if palette_store.len() < 255 {
                let id = palette_store.len() as u8;
                palette_output.push(red);
                palette_output.push(green);
                palette_output.push(blue);
                palette_store.insert(key, id);
                id
            } else {
                println!("[include_image] too many colours for {path} to do an exact palette256");
                return None;
            };

            pixel_output.push(palette_id);

            if has_alpha {
                let alpha = pixel[3];
                pixel_output.push(alpha);
            }
        }
    }

    let mut output = Vec::with_capacity(1 + palette_output.len() + pixel_output.len());
    println!("[include_image] {path} has {} colours", palette_store.len());
    output.push(palette_store.len() as u8);
    output.extend_from_slice(&palette_output);

    if palette_store.len() == 1 {
        // If we only have one colour, we can strip out the index
        println!(
            "[include_image] {path} single colour palette, stripping out palette ref (start with {})",
            pixel_output.len()
        );
        pixel_output.iter().enumerate().for_each(|(offset, value)| {
            if (offset % 2) == 1 {
                output.push(*value);
            }
        });
        println!(
            "[include_image] {path} saved {} bytes",
            pixel_output.len() - (output.len() - palette_store.len() - 1)
        );
    } else {
        output.extend_from_slice(&pixel_output);
    }

    Some(output)
}

fn path_to_image(args: &ParsedArgs) -> (Vec<u8>, String, String, usize, usize, bool) {
    let path = args.path.as_str();
    let (has_alpha, img) = match image::open(path) {
        Ok(img) => (img.color().has_alpha(), img),
        Err(e) => panic!("failed to open image {}: {}", path, e),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;
    let rgba = img.to_rgba8();

    let (struct_name, bytes) = {
        let pixel = to_rgb565a_with_alpha(&rgba, width, height, has_alpha);
        let palette = to_rgba_palette_with_alpha(path, &rgba, width, height, has_alpha)
            .unwrap_or_else(|| {
                let incoming_value = args.force_rgb.unwrap_or(8);

                to_rgba_palette256_with_alpha(
                    path,
                    &rgba,
                    width,
                    height,
                    has_alpha,
                    // this ensures that the paletter is between 4 and 256 colours
                    incoming_value.min(8).max(2),
                )
            });

        if let Some(palette_size) = args.force_rgb
            && palette_size == 0
        {
            ("ImageRGB565A".to_string(), pixel)
        } else if palette.len() < pixel.len() {
            println!(
                "[include_image] using palette based image for {path} - saved {} bytes ({} palette vs {} rgb565)",
                pixel.len() - palette.len(),
                palette.len(),
                pixel.len()
            );

            ("ImageRGBP256".to_string(), palette)
        } else {
            ("ImageRGB565A".to_string(), pixel)
        }
    };

    let path = path
        .split('/')
        .next_back()
        .expect("failed to get last part of path");
    let split: Vec<_> = path.split('.').collect();
    let name = remove_non_alphanumeric(&split[0..split.len() - 1].join(".")).to_uppercase();

    (bytes, struct_name, name, width, height, has_alpha)
}

struct ParsedArgs {
    path: String,
    force_rgb: Option<u32>,
}

impl Parse for ParsedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let force_rgb = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let palette_size: syn::LitInt = input.parse()?;
            palette_size.base10_parse().ok()
        } else {
            Some(8)
        };
        let path = path.value();
        Ok(ParsedArgs { path, force_rgb })
    }
}

#[proc_macro]
pub fn include_image(input: TokenStream) -> TokenStream {
    // parse the input into a comma separated list of arguments
    let parsed_args = parse_macro_input!(input as ParsedArgs);
    let (pixel_bytes, struct_name, name, width, height, has_alpha) = path_to_image(&parsed_args);

    let byte_array = pixel_bytes.as_slice();
    let byte_count = byte_array.len();

    let name_ident = syn::Ident::new(&name, Span::call_site().into());
    let struct_ident = syn::Ident::new(&struct_name, Span::call_site().into());

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let id = hasher.finish() as u32;

    let byte_tokens = pixel_bytes
        .iter()
        .map(|b| quote! { #b })
        .collect::<Vec<_>>();

    let output = quote! {
        pub const #name_ident: ::include_image::#struct_ident<#byte_count> = ::include_image::#struct_ident {
            id: #id,
            width: #width,
            height: #height,
            has_alpha: #has_alpha,
            pixels: [#(#byte_tokens),*],
        };
    };

    output.into()
}
