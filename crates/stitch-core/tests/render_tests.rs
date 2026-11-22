mod helpers;

use helpers::{pixel_at, solid_image};
use stitch_core::render::render;
use stitch_core::{
    compose, Alignment, CanvasPlan, ComposeOptions, Divider, Frame, Insets, LayoutKind, PaneStyle,
    PlacedImage, ResizeMode, RgbaColor, Spacing,
};

#[test]
fn renders_background_and_stacked_pixels() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(1),
        padding: Insets::all(1),
        background: RgbaColor {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(1),
        padding: Insets::all(1),
        background: RgbaColor {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(1),
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameHeight,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::all(0),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameWidth,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor::TRANSPARENT,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor {
            r: 255,
            g: 255,
            b: 255,
            a: 19,
        },
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let composed = compose(&images, &options).expect("compose low-alpha");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 255, 255, 25]
    );
}

fn red_and_filler(size: u32) -> Vec<stitch_core::InputImage> {
    vec![
        solid_image(size, size, [255, 0, 0, 255]),
        solid_image(size, size, [0, 255, 0, 255]),
    ]
}

fn pane_options(pane_style: PaneStyle, background: RgbaColor) -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style,
        divider: None,
        column_weights: Vec::new(),
    }
}

#[test]
fn default_pane_style_keeps_existing_output() {
    let images = vec![
        solid_image(2, 1, [255, 0, 0, 255]),
        solid_image(2, 1, [0, 255, 0, 255]),
    ];
    let plain = ComposeOptions::default();
    let explicit = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle {
            frame: None,
            corner_radius: 0,
        },
        divider: None,
        column_weights: Vec::new(),
    };

    let composed = compose(&images, &plain).expect("compose undecorated");
    let styled = compose(&images, &explicit).expect("compose explicit style");

    assert_eq!(composed, styled);
}

#[test]
fn corner_radius_masks_corner_pixels() {
    let images = red_and_filler(4);
    let options = pane_options(
        PaneStyle {
            frame: None,
            corner_radius: 2,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose rounded pane");

    for corner in [(0, 0), (3, 0), (0, 3), (3, 3)] {
        assert_eq!(
            pixel_at(&composed.pixels, composed.width, corner.0, corner.1),
            [255, 255, 255, 255],
            "corner {:?} should show the background",
            corner
        );
    }

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 1),
        [255, 0, 0, 255]
    );
}

#[test]
fn corner_radius_is_symmetric() {
    let images = red_and_filler(6);
    let options = pane_options(
        PaneStyle {
            frame: None,
            corner_radius: 3,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose fully rounded pane");

    for corner in [(0, 0), (5, 0), (0, 5), (5, 5)] {
        assert_eq!(
            pixel_at(&composed.pixels, composed.width, corner.0, corner.1),
            [255, 255, 255, 255],
            "corner {:?} should show the background",
            corner
        );
    }

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 3),
        [255, 0, 0, 255]
    );
}

#[test]
fn corner_radius_masks_all_four_corners() {
    let images = red_and_filler(4);
    let options = pane_options(
        PaneStyle {
            frame: None,
            corner_radius: 1,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose rounded pane");

    for corner in [(0, 0), (3, 0), (0, 3), (3, 3)] {
        assert_eq!(
            pixel_at(&composed.pixels, composed.width, corner.0, corner.1),
            [255, 255, 255, 255],
            "corner {:?} should show the background",
            corner
        );
    }

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 1),
        [255, 0, 0, 255]
    );
}

#[test]
fn frame_draws_ring_inside_pane() {
    let images = red_and_filler(5);
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 1,
                color: RgbaColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            }),
            corner_radius: 0,
        },
        RgbaColor::TRANSPARENT,
    );

    let composed = compose(&images, &options).expect("compose framed pane");

    for edge in [(0, 0), (4, 4), (2, 0)] {
        assert_eq!(
            pixel_at(&composed.pixels, composed.width, edge.0, edge.1),
            [0, 0, 0, 255],
            "ring pixel {:?} should be the frame color",
            edge
        );
    }

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 2),
        [255, 0, 0, 255]
    );
}

#[test]
fn frame_alpha_composites_with_background() {
    let images = red_and_filler(3);
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 1,
                color: RgbaColor {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 128,
                },
            }),
            corner_radius: 0,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose framed pane");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [127, 127, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
}

#[test]
fn corner_radius_is_clamped() {
    let images = red_and_filler(3);
    let options = pane_options(
        PaneStyle {
            frame: None,
            corner_radius: 100,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose over-clamped pane");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 255, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 1, 1),
        [255, 0, 0, 255]
    );
}

#[test]
fn frame_covers_pane_when_frame_exceeds_size() {
    let images = red_and_filler(3);
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 5,
                color: RgbaColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            }),
            corner_radius: 0,
        },
        RgbaColor::WHITE,
    );

    let composed = compose(&images, &options).expect("compose oversized frame");

    for y in 0..3 {
        for x in 0..3 {
            assert_eq!(
                pixel_at(&composed.pixels, composed.width, x, y),
                [0, 0, 0, 255],
                "({},{}) should be frame color",
                x,
                y
            );
        }
    }
}

fn before_after_options(spacing: u32, padding: u32, divider: Option<Divider>) -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(spacing),
        padding: Insets::all(padding),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider,
        column_weights: Vec::new(),
    }
}

fn blue_and_green(size: u32) -> Vec<stitch_core::InputImage> {
    vec![
        solid_image(size, size, [0, 0, 255, 255]),
        solid_image(size, size, [0, 255, 0, 255]),
    ]
}

#[test]
fn divider_is_centered_in_the_gap() {
    let images = blue_and_green(2);
    let options = before_after_options(
        4,
        0,
        Some(Divider {
            width: 2,
            color: RgbaColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }),
    );

    let composed = compose(&images, &options).expect("compose with divider");

    assert_eq!(composed.width, 8);
    assert_eq!(composed.height, 2);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 4, 1),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 2, 0),
        [255, 255, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 5, 1),
        [255, 255, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [0, 0, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 7, 1),
        [0, 255, 0, 255]
    );
}

#[test]
fn divider_spans_pane_height() {
    let images = blue_and_green(2);
    let options = before_after_options(
        4,
        3,
        Some(Divider {
            width: 2,
            color: RgbaColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }),
    );

    let composed = compose(&images, &options).expect("compose padded divider");

    assert_eq!(composed.width, 14);
    assert_eq!(composed.height, 8);

    for row in [3, 4] {
        for column in [6, 7] {
            assert_eq!(
                pixel_at(&composed.pixels, composed.width, column, row),
                [255, 0, 0, 255],
                "({column},{row}) should be divider",
            );
        }
    }

    for row in [0, 1, 2, 5, 6, 7] {
        for column in [6, 7] {
            assert_eq!(
                pixel_at(&composed.pixels, composed.width, column, row),
                [255, 255, 255, 255],
                "({column},{row}) should be background",
            );
        }
    }
}

#[test]
fn divider_color_alpha_composites() {
    let images = blue_and_green(2);
    let options = before_after_options(
        4,
        0,
        Some(Divider {
            width: 2,
            color: RgbaColor {
                r: 0,
                g: 0,
                b: 0,
                a: 128,
            },
        }),
    );

    let composed = compose(&images, &options).expect("compose alpha divider");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 0),
        [127, 127, 127, 255]
    );
}

#[test]
fn default_divider_keeps_output_identical() {
    let images = blue_and_green(2);
    let none = before_after_options(4, 0, None);
    let zero_width = before_after_options(
        4,
        0,
        Some(Divider {
            width: 0,
            color: RgbaColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }),
    );

    let composed = compose(&images, &none).expect("compose without divider");
    let zero = compose(&images, &zero_width).expect("compose zero-width divider");

    assert_eq!(composed, zero);
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 3, 0),
        [255, 255, 255, 255]
    );
}

#[test]
fn global_style_applies_when_all_styles_are_none() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 1,
                color: RgbaColor::WHITE,
            }),
            corner_radius: 0,
        },
        RgbaColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
    );

    let composed = compose(&images, &options).expect("compose global frames");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 255, 255, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 4),
        [255, 255, 255, 255]
    );
}

#[test]
fn per_image_style_replaces_global_style() {
    let mut images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    images[0].style = Some(PaneStyle::default());
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 1,
                color: RgbaColor::WHITE,
            }),
            corner_radius: 0,
        },
        RgbaColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
    );

    let composed = compose(&images, &options).expect("compose mixed styles");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 4),
        [255, 255, 255, 255]
    );
}

#[test]
fn per_image_style_adds_decoration_to_one_pane() {
    let mut images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    images[1].style = Some(PaneStyle {
        frame: Some(Frame {
            width: 1,
            color: RgbaColor::WHITE,
        }),
        corner_radius: 0,
    });
    let options = pane_options(
        PaneStyle::default(),
        RgbaColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
    );

    let composed = compose(&images, &options).expect("compose one decorated pane");

    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 0),
        [255, 0, 0, 255]
    );
    assert_eq!(
        pixel_at(&composed.pixels, composed.width, 0, 4),
        [255, 255, 255, 255]
    );
}

#[test]
fn none_styles_match_explicit_global_styles_byte_for_byte() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(2, 6, [0, 255, 0, 255]),
    ];
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame {
                width: 1,
                color: RgbaColor::WHITE,
            }),
            corner_radius: 2,
        },
        RgbaColor::WHITE,
    );

    let mut explicit = images.clone();
    for image in &mut explicit {
        image.style = Some(options.pane_style);
    }

    let inherited = compose(&images, &options).expect("compose inherited styles");
    let overridden = compose(&explicit, &options).expect("compose explicit global styles");

    assert_eq!(inherited, overridden);
}
