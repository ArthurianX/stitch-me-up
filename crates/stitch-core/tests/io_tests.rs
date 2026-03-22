mod helpers;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use stitch_core::{load_image_from_path, save_png, ComposedImage};

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
