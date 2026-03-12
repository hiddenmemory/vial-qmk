// This is a modification of this: https://github.com/not-nullptr/qmk-rs/tree/master/rust

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use image::{ImageBuffer, Rgb, Rgba};
use proc_macro::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{parse::Parse, parse_macro_input};

fn remove_non_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
    re.replace_all(input, "").to_string()
}

fn to_rgb565a_with_alpha(
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
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
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
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
                println!("[include_image] too many colours for {path}");
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
    output.extend_from_slice(&pixel_output);

    Some(output)
}

fn path_to_image(path: &str) -> (Vec<u8>, String, String, usize, usize, bool) {
    let (has_alpha, img) = match image::open(path) {
        Ok(img) => (img.color().has_alpha(), img),
        Err(e) => panic!("failed to open image {}: {}", path, e),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;

    let (struct_name, bytes) = {
        let pixel = to_rgb565a_with_alpha(img.to_rgba8(), width, height, has_alpha);
        let palette = to_rgba_palette256_with_alpha(path, img.to_rgba8(), width, height, has_alpha);

        if let Some(palette) = palette
            && palette.len() < pixel.len()
        {
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
}

impl Parse for ParsedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let path = path.value();
        Ok(ParsedArgs { path })
    }
}

#[proc_macro]
pub fn include_image(input: TokenStream) -> TokenStream {
    // parse the input into a comma separated list of arguments
    let parsed_args = parse_macro_input!(input as ParsedArgs);
    let (pixel_bytes, struct_name, name, width, height, has_alpha) =
        path_to_image(&parsed_args.path);

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
