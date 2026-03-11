// This is a modification of this: https://github.com/not-nullptr/qmk-rs/tree/master/rust

use std::hash::{DefaultHasher, Hash, Hasher};

use image::{ImageBuffer, Rgb, Rgba};
use proc_macro::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{parse::Parse, parse_macro_input};

fn remove_non_alphanumeric(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
    re.replace_all(input, "").to_string()
}

fn to_format(img: ImageBuffer<Rgb<u8>, Vec<u8>>, width: usize, height: usize) -> Vec<u8> {
    let mut pixel_output = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];

            pixel_output.push((red & 0xF8) | (green >> 5));
            pixel_output.push((green & 0b11111100) << 3 | (blue >> 3));
        }
    }

    pixel_output
}

fn to_format_a(img: ImageBuffer<Rgba<u8>, Vec<u8>>, width: usize, height: usize) -> Vec<u8> {
    let mut pixel_output = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];
            let alpha = pixel[3];

            pixel_output.push((red & 0xF8) | (green >> 5));
            pixel_output.push((green & 0b11111100) << 3 | (blue >> 3));
            pixel_output.push(alpha);
        }
    }

    pixel_output
}

fn path_to_image(path: &str) -> (Vec<u8>, String, usize, usize, bool) {
    let (has_alpha, img) = match image::open(path) {
        Ok(img) => (img.color().has_alpha(), img),
        Err(e) => panic!("failed to open image {}: {}", path, e),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;

    let bytes = if has_alpha {
        to_format_a(img.to_rgba8(), width, height)
    } else {
        to_format(img.to_rgb8(), width, height)
    };

    let path = path
        .split('/')
        .next_back()
        .expect("failed to get last part of path");
    let split: Vec<_> = path.split('.').collect();
    let name = remove_non_alphanumeric(&split[0..split.len() - 1].join(".")).to_uppercase();

    (bytes, name, width, height, has_alpha)
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
    let (pixel_bytes, name, width, height, has_alpha) = path_to_image(&parsed_args.path);

    let width = width as u8;
    let height = height as u8;

    let byte_array = pixel_bytes.as_slice();
    let byte_count = byte_array.len();

    let name_ident = syn::Ident::new(&name, Span::call_site().into());

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let id = hasher.finish() as u32;

    let byte_tokens = pixel_bytes
        .iter()
        .map(|b| quote! { #b })
        .collect::<Vec<_>>();

    let output = quote! {
        pub const #name_ident: ::include_image::ImageBRG565A<#byte_count> = ::include_image::ImageBRG565A {
            id: #id,
            width: #width,
            height: #height,
            has_alpha: #has_alpha,
            pixels: [#(#byte_tokens),*],
        };
    };

    output.into()
}
