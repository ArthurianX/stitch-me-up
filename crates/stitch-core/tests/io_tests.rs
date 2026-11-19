mod helpers;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use helpers::solid_image;
use stitch_core::{
    compose_to_png, encode_png, load_image_from_bytes, load_image_from_path, save_png, Alignment,
    ComposeError, ComposeOptions, ComposedImage, HexColorError, InputImage, Insets, LayoutKind,
    PaneStyle, ResizeMode, RgbaColor,
};

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("stitch-me-up-{nanos}-{name}"))
}

#[test]
fn saves_png_and_loads_it_back() {
    let path = temp_path("roundtrip.png");
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255, 0, 255, 0, 255],
    };

    save_png(&path, &image).expect("save png");
    let loaded = load_image_from_path(&path).expect("load png");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels, image.pixels);

    let _ = fs::remove_file(path);
}

#[test]
fn loads_jpeg_input() {
    let path = temp_path("source.jpg");
    let buffer = image::RgbImage::from_raw(2, 1, vec![200, 100, 50, 25, 75, 125]).unwrap();
    buffer
        .save_with_format(&path, image::ImageFormat::Jpeg)
        .expect("write jpeg");

    let loaded = load_image_from_path(&path).expect("load jpeg");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels.len(), 8);

    let _ = fs::remove_file(path);
}

fn png_bytes(image: &InputImage) -> Vec<u8> {
    let composed = ComposedImage {
        width: image.width,
        height: image.height,
        pixels: image.pixels.clone(),
    };

    encode_png(&composed).expect("encode png")
}

fn vertical_options() -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 1,
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    }
}

#[test]
fn encodes_png_in_memory_and_loads_it_back() {
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255, 0, 255, 0, 255],
    };

    let encoded = encode_png(&image).expect("encode png");
    let loaded = load_image_from_bytes(&encoded).expect("load encoded png");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels, image.pixels);
}

#[test]
fn compose_to_png_round_trips() {
    let red = solid_image(2, 1, [255, 0, 0, 255]);
    let green = solid_image(2, 1, [0, 255, 0, 255]);

    let output = compose_to_png(&[png_bytes(&red), png_bytes(&green)], &vertical_options())
        .expect("compose to png");

    let composed = load_image_from_bytes(&output).expect("load composed png");

    assert_eq!(composed.width, 2);
    assert_eq!(composed.height, 3);
    assert_eq!(&composed.pixels[0..4], &[255, 0, 0, 255]);
    assert_eq!(&composed.pixels[4..8], &[255, 0, 0, 255]);
    assert_eq!(&composed.pixels[8..12], &[255, 255, 255, 255]);
    assert_eq!(&composed.pixels[16..20], &[0, 255, 0, 255]);
}

#[test]
fn rejects_invalid_image_bytes() {
    let error = load_image_from_bytes(b"not an image").unwrap_err();
    assert!(matches!(error, ComposeError::ImageIo(_)));

    let good = png_bytes(&solid_image(2, 1, [255, 0, 0, 255]));
    let error = compose_to_png(&[good, b"not an image".to_vec()], &vertical_options()).unwrap_err();
    assert!(matches!(error, ComposeError::ImageIo(_)));
}

#[test]
fn parse_hex_color_variants() {
    let expected_rgb = RgbaColor {
        r: 0xaa,
        g: 0xbb,
        b: 0xcc,
        a: 0xff,
    };

    assert_eq!(RgbaColor::parse_hex("#aabbcc").unwrap(), expected_rgb);
    assert_eq!(RgbaColor::parse_hex("aabbcc").unwrap(), expected_rgb);

    let expected_rgba = RgbaColor {
        r: 0x01,
        g: 0x02,
        b: 0x03,
        a: 0x04,
    };

    assert_eq!(RgbaColor::parse_hex("#01020304").unwrap(), expected_rgba);
    assert_eq!(RgbaColor::parse_hex("01020304").unwrap(), expected_rgba);

    assert_eq!(
        RgbaColor::parse_hex("#12345").unwrap_err(),
        HexColorError::InvalidLength
    );
    assert_eq!(
        HexColorError::InvalidLength.to_string(),
        "expected color in #RRGGBB or #RRGGBBAA format"
    );
    assert_eq!(
        RgbaColor::parse_hex("#gg0000").unwrap_err(),
        HexColorError::InvalidComponent
    );
    assert_eq!(
        HexColorError::InvalidComponent.to_string(),
        "invalid hex color component"
    );
}
