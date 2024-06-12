// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

extern crate bytes;

mod bytearray;

use std::io::Write;

use ab_glyph::{FontRef, PxScale};
pub use bytearray::ByteArray;
use imageproc::{
    drawing::{draw_text_mut, text_size},
    image::{imageops::overlay, ImageBuffer, Rgb, RgbImage},
};
use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(Clone)]
pub enum OldData {
    String(String),
    Bool(bool),
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
}

pub fn encode_xml(xml: String) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let _ = encoder.write_all(xml.as_bytes());
    encoder.finish()
}

pub fn generate_captcha(len: usize) -> String {
    let one_char = || CHARSET[rand::thread_rng().gen_range(0..CHARSET.len())] as char;
    std::iter::repeat_with(one_char).take(len).collect()
}

pub fn generate_captcha_image(code: &str) -> (ImageBuffer<Rgb<u8>, Vec<u8>>, u32, u32) {
    let font = FontRef::try_from_slice(include_bytes!("../DejaVuSans.ttf")).unwrap();

    let height = 15.0;
    let scale = PxScale {
        x: height * 1.15,
        y: height,
    };
    let (w, h) = text_size(scale, &font, code);

    let mut image = RgbImage::new(w + 5, 17);
    draw_text_mut(
        &mut image,
        Rgb([255u8, 255u8, 255u8]),
        2,
        1,
        scale,
        &font,
        code,
    );

    let mut final_image = RgbImage::new(w + 5, 17);
    overlay(&mut final_image, &image, -1, 2);
    overlay(&mut final_image, &image, 0, 2);
    overlay(&mut final_image, &image, -1, 3);
    overlay(&mut final_image, &image, -1, 1);
    overlay(&mut final_image, &image, 0, 3);
    overlay(&mut final_image, &image, -2, 1);
    overlay(&mut final_image, &image, -2, 3);
    overlay(&mut final_image, &image, 0, 1);

    (final_image, w, h)
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    bytes.into_iter().map(|b| *b as char).collect::<String>()
}

pub fn str_to_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .into_iter()
        .map(|c| c as u8)
        .collect::<Vec<u8>>()
}

pub fn language_code(code: &str) -> &str {
    match code {
        "afrikaans" => "af",
        "azərbaycan dili" => "az",
        "bahasa indonesia" => "id",
        "bahasa melayu" => "ms",
        "bislama" => "bi",
        "português brasileiro" => "br",
        "bosanski jezik" => "bs",
        "català" => "ca",
        "chicheŵa" => "ny",
        "dansk" => "da",
        "deutsch" => "de",
        "eesti keel" => "et",
        "español" => "es",
        "ekakairũ naoero" => "na",
        // TODO: add all langues
        "english" | _ => "en",
    }
}

pub fn language_id(lang: &str) -> i8 {
    match lang {
        // TODO: add all langues
        "en" | _ => 1,
    }
}

pub fn language_info(lang: &str) -> (&str, &str) {
    match lang {
        "af" => ("Afrikaans", "za"),
        "az" => ("Azərbaycan dili", "az"),
        "id" => ("Bahasa Indonesia", "id"),
        "ms" => ("Bahasa Melayu", "my"),
        "bi" => ("Bislama", "vu"),
        "br" => ("Português brasileiro", "br"),
        "bs" => ("Bosanski jezik", "ba"),
        "ca" => ("Català", "ad"),
        "ny" => ("ChiCheŵa", "mw"),
        "da" => ("Dansk", "dk"),
        "de" => ("Deutsch", "de"),
        "et" => ("Eesti keel", "ee"),
        "es" => ("Español", "es"),
        "na" => ("Ekakairũ Naoero", "nr"),
        // TODO: add all langues
        "en" | _ => ("English", "gb"),
    }
}

pub fn language_list<'a>() -> Vec<&'a str> {
    vec![
        "af", "az", "id", "ms", "bi", "br", "bs", "ca", "ny", "da", "de", "et", "es", "na", "en",
    ]
}
