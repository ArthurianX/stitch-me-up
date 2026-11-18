mod helpers;

use helpers::{pixel_at, solid_image};
use stitch_core::{compose, Alignment, ComposeOptions, Insets, LayoutKind, ResizeMode, RgbaColor};

#[test]
fn renders_background_and_stacked_pixels() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 1,
        padding: Insets::all(1),
        background: RgbaColor {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose image");

    assert_eq!(composed.width, 3);
    assert_eq!(composed.height, 5);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [10, 20, 30, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 2),
        [10, 20, 30, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 3),
        [0, 255, 0, 255]
    );
}

#[test]
fn renders_grid_cells_and_background() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
        solid_image(1, 1, [0, 0, 255, 255]),
        solid_image(1, 1, [255, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 2 },
        spacing: 1,
        padding: Insets::all(1),
        background: RgbaColor {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose grid");

    assert_eq!(composed.width, 5);
    assert_eq!(composed.height, 5);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 1),
        [0, 255, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 3),
        [0, 0, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 3),
        [255, 255, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 2),
        [10, 20, 30, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [10, 20, 30, 255]
    );
}

#[test]
fn centers_smaller_images_on_vertical_layout() {
    let images = vec![
        solid_image(3, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose centered");

    assert_eq!(composed.width, 3);
    assert_eq!(composed.height, 2);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 1),
        [0, 0, 0, 0]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [0, 255, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 1),
        [0, 0, 0, 0]
    );
}

#[test]
fn applies_fit_to_same_height_during_rendering() {
    let images = vec![
        solid_image(2, 2, [255, 0, 0, 255]),
        solid_image(1, 2, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::HorizontalStack,
        spacing: 0,
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameHeight,
    };

    let composed = compose(&images, &options).expect("compose same-height");

    assert_eq!(composed.width, 3);
    assert_eq!(composed.height, 2);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 1),
        [0, 255, 0, 255]
    );
}

#[test]
fn applies_fit_to_same_width_during_rendering() {
    let images = vec![
        solid_image(2, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 0, 255, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameWidth,
    };

    let composed = compose(&images, &options).expect("compose same-width");

    assert_eq!(composed.width, 2);
    assert_eq!(composed.height, 3);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 2),
        [0, 0, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 2),
        [0, 0, 255, 255]
    );
}
