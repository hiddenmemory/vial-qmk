// This is a modification of this: https://github.com/not-nullptr/qmk-rs/tree/master/rust

use std::{
    collections::HashMap,
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
};

use fontdue::{
    Font,
    layout::{CoordinateSystem, Layout, TextStyle},
};
use image::{ImageBuffer, Rgba};
use proc_macro::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::{Token, parenthesized, parse::Parse, parse_macro_input};

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
                    incoming_value.clamp(2, 8),
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

fn id_from_str(value: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as u32
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

    let id = id_from_str(&name);

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

struct FontParsedArgs {
    path: String,
    size: u8,
    chars: String,
    name_ident: Option<syn::Ident>,
}

impl Parse for FontParsedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let size: syn::LitInt = input.parse()?;
        input.parse::<Token![,]>()?;

        let tag = format!("{}:{}", path.value(), size);

        let chars = if input.peek(syn::LitStr) {
            let chars: syn::LitStr = input.parse()?;
            println!(
                "[{tag}] generating custom character table: {}",
                chars.value()
            );
            chars.value()
        } else if input.peek(syn::Ident) {
            let _ident = input.parse::<syn::Ident>()?;
            let ident = _ident.to_string();

            println!("[{tag}] generating for {_ident}");

            let mut glyphs = String::new();

            if ident.eq("ASCII") {
                let ascii = (0x21..0x7F_u32)
                    .flat_map(char::from_u32)
                    .collect::<String>();

                glyphs.push_str(&ascii);
                println!("[{tag}] → including ASCII table");
            }

            if (ident.eq("ASCII") || ident.eq("unicode")) && input.peek(syn::token::Paren) {
                let inner;
                parenthesized!(inner in input);
                let additional_chars: syn::LitStr = inner.parse()?;
                glyphs.push_str(&additional_chars.value());
                println!(
                    "[{tag}] → including unicode table: {}",
                    additional_chars.value()
                );
            }

            glyphs
        } else {
            "".to_string()
        };

        let path = path.value();
        let size = size.base10_parse()?;

        let name_ident = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(FontParsedArgs {
            path,
            size,
            chars,
            name_ident,
        })
    }
}

#[proc_macro]
pub fn include_font(input: TokenStream) -> TokenStream {
    let mut parsed_args = parse_macro_input!(input as FontParsedArgs);

    let path = parsed_args
        .path
        .split('/')
        .next_back()
        .expect("failed to get last part of path");

    let split: Vec<_> = path.split('.').collect();

    let name = format!(
        "{}_{}",
        remove_non_alphanumeric(&split[0..split.len() - 1].join(".")).to_uppercase(),
        parsed_args.size
    );

    let name_ident = parsed_args
        .name_ident
        .take()
        .unwrap_or(syn::Ident::new(&name, Span::call_site().into()));

    let mut font: Vec<u8> = vec![];

    File::open(&parsed_args.path)
        .expect("open font file")
        .read_to_end(&mut font)
        .expect("read font file");

    let font = Font::from_bytes(font.as_slice(), Default::default()).expect("create text renderer");

    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

    layout.append(
        &[&font],
        &TextStyle::new(&parsed_args.chars, parsed_args.size as f32, 0),
    );

    // println!("glyphs = >{}<", parsed_args.chars);

    // println!(
    //     "{:#?}",
    //     font.horizontal_line_metrics(parsed_args.size as f32)
    // );

    // println!("{:#?}", layout.glyphs());
    // println!("{:#?}", layout.lines());
    // println!("{:#?}", layout.height());

    let glyphs_with_data = layout
        .glyphs()
        .iter()
        .map(|position| {
            let (metrics, data) = font.rasterize_indexed(position.key.glyph_index, position.key.px);
            (position, metrics, data)
        })
        .collect::<Vec<_>>();

    // - Create a buffer that is (glyphs_with_data.last().{x + width}, layout.lines().first().max_new_line_size)
    // - Go through each glyphs_with_data and put them in the buffer in the correct byte location

    let max_height = layout
        .lines()
        .expect("Lines withing the layout")
        .first()
        .expect("At least one line")
        .max_new_line_size as usize;

    // This checks to see if we have anything hanging down over the max_height reported from lines
    let max_height = glyphs_with_data
        .iter()
        .map(|(p, _, _)| p.y as usize + p.height)
        .max()
        .unwrap_or(max_height)
        .max(max_height);

    let (last_glyph_position, _, _) = glyphs_with_data.last().expect("At least one glyph");
    let max_width = (last_glyph_position.x as usize) + last_glyph_position.width;

    // println!(
    //     "width = {max_width}, height = {max_height}, buffer: {}",
    //     max_width * max_height
    // );

    let mut bytes = vec![0u8; max_width * max_height];
    let mut code_points: Vec<(char, usize, usize)> = Vec::with_capacity(glyphs_with_data.len());

    for (position, metrics, data) in glyphs_with_data {
        // println!(
        //     "{name} processing glyph = '{}' m.w = {} m.h = {} p.x = {} p.y = {}",
        //     position.parent, metrics.width, metrics.height, position.x, position.y
        // );

        for data_y in 0..metrics.height {
            for data_x in 0..metrics.width {
                let data_offset = (data_y * metrics.width) + data_x;

                let byte_x = position.x as usize + data_x;
                let byte_y = position.y as usize + data_y;

                let byte_offset = (byte_y * max_width) + byte_x;

                // println!(
                //     "{name} {} -> dx = {data_x}, dy = {data_y}, bx = {byte_x} by={byte_y} do = {data_offset}, bo = {byte_offset}, mw = {max_width} mh = {max_height}",
                //     position.parent
                // );

                bytes[byte_offset] = data[data_offset];
            }
        }

        code_points.push((position.parent, position.x as usize, position.width));
    }

    let id = id_from_str(&name);
    let font_size = parsed_args.size;
    let (space_width, character_padding) = {
        let mut space_layout = Layout::new(CoordinateSystem::PositiveYDown);
        space_layout.append(
            &[&font],
            &TextStyle::new("H HH", parsed_args.size as f32, 0),
        );
        let glyphs = space_layout.glyphs();
        (
            (glyphs[2].x - glyphs[1].x) as u8,
            (glyphs[3].x - (glyphs[2].x + glyphs[2].width as f32)) as u8,
        )
    };
    let count = parsed_args.chars.len();
    let has_alpha = true;

    let byte_count = bytes.len();
    let code_point_count = code_points.len();
    let code_point_tokens = code_points
        .iter()
        .map(|(code_point, x, width)| quote! { ( #code_point, #x, #width ) })
        .collect::<Vec<_>>();
    let byte_tokens = bytes.iter().map(|b| quote! { #b }).collect::<Vec<_>>();

    println!("{name}: size {max_width}, {max_height} for {count} characters");

    let output = quote! {
        pub const #name_ident: ::include_image::Font<#code_point_count, #byte_count> = ::include_image::Font {
            id: #id,
            font_size: #font_size,
            space_width: #space_width,
            character_padding: #character_padding,
            count: #count,
            width: #max_width,
            height: #max_height,
            has_alpha: #has_alpha,
            code_points: [#(#code_point_tokens),*],
            pixels: [#(#byte_tokens),*],
        };
    };

    output.into()
}
