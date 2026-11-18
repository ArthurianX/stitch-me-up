mod helpers;

use helpers::{pixel_at, solid_image};
use stitch_core::render::render;
use stitch_core::{
    compose, Alignment, CanvasPlan, ComposeOptions, Insets, LayoutKind, PlacedImage, ResizeMode,
    RgbaColor,
};

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
fn renders_before_after_with_equal_pane_heights() {
    let images = vec![
        solid_image(2, 2, [255, 0, 0, 255]),
        solid_image(1, 4, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 1,
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose before-after");

    assert_eq!(composed.width, 6);
    assert_eq!(composed.height, 4);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 3),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 5, 0),
        [0, 255, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 5, 3),
        [0, 255, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 4, 0),
        [0, 0, 0, 0]
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

#[test]
fn blends_semi_transparent_source_over_background() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 128]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose blended");

    assert_eq!(composed.width, 1);
    assert_eq!(composed.height, 2);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [128, 0, 127, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 1),
        [0, 255, 0, 255]
    );
}

#[test]
fn fully_transparent_pixels_leave_background() {
    let images = vec![
        solid_image(1, 1, [10, 20, 30, 0]),
        solid_image(1, 1, [0, 0, 0, 0]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose transparent");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [1, 2, 3, 4]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 1),
        [1, 2, 3, 4]
    );
}

#[test]
fn semi_transparent_source_over_transparent_background_keeps_color() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 128]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose over transparent background");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 128]
    );
}

#[test]
fn overlapping_semi_transparent_planes_blend() {
    let images = vec![
        solid_image(1, 1, [0, 0, 255, 255]),
        solid_image(1, 1, [255, 0, 0, 128]),
    ];
    let plan = CanvasPlan {
        width: 1,
        height: 1,
        placed_images: vec![
            PlacedImage {
                index: 0,
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            PlacedImage {
                index: 1,
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        ],
    };
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
    };

    let composed = render(&images, &plan, &options).expect("render overlapping planes");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [128, 0, 127, 255]
    );
}

#[test]
fn low_alpha_blend_uses_exact_weighting() {
    let images = vec![
        solid_image(1, 1, [255, 255, 255, 6]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor {
            r: 255,
            g: 255,
            b: 255,
            a: 19,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
    };

    let composed = compose(&images, &options).expect("compose low-alpha");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 255, 255, 25]
    );
}
