#![allow(dead_code)]

use stitch_core::InputImage;

pub fn solid_image(width: u32, height: u32, color: [u8; 4]) -> InputImage {
    let mut pixels = Vec::new();
    for _ in 0..(width * height) {
        pixels.extend_from_slice(&color);
    }

    InputImage::new(width, height, pixels)
}

pub fn pixel_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let start = ((y * width + x) * 4) as usize;
    [
        pixels[start],
        pixels[start + 1],
        pixels[start + 2],
        pixels[start + 3],
    ]
}

pub fn striped_image(width: u32, height: u32, colors: &[[u8; 4]]) -> InputImage {
    let mut pixels = Vec::new();
    for _ in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&colors[(x as usize) % colors.len()]);
        }
    }
    InputImage::new(width, height, pixels)
}
