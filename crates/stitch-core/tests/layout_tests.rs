mod helpers;

use helpers::solid_image;
use stitch_core::{
    plan_layout, Alignment, ComposeError, ComposeOptions, Divider, Insets, LayoutKind, PaneStyle,
    ResizeMode, RgbaColor,
};

#[test]
fn rejects_fewer_than_two_images() {
    let images = vec![solid_image(1, 1, [0, 0, 0, 255])];

    let error = plan_layout(&images, &ComposeOptions::default()).unwrap_err();

    assert!(matches!(error, ComposeError::NotEnoughImages));
}

#[test]
fn rejects_invalid_pixel_buffer_lengths() {
    let mut image = solid_image(1, 1, [0, 0, 0, 255]);
    image.pixels.pop();
    let images = vec![solid_image(1, 1, [255, 0, 0, 255]), image];

    let error = plan_layout(&images, &ComposeOptions::default()).unwrap_err();

    assert!(matches!(
        error,
        ComposeError::InvalidPixelBuffer {
            index: 1,
            expected: 4,
            actual: 3,
        }
    ));
}

#[test]
fn plans_vertical_stack_with_spacing_padding_and_center_alignment() {
    let images = vec![
        solid_image(10, 4, [255, 0, 0, 255]),
        solid_image(6, 8, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 3,
        padding: Insets {
            top: 2,
            right: 4,
            bottom: 6,
            left: 8,
        },
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("vertical plan");

    assert_eq!(plan.width, 22);
    assert_eq!(plan.height, 23);
    assert_eq!(plan.placed_images.len(), 2);
    assert_eq!(plan.placed_images[0].x, 8);
    assert_eq!(plan.placed_images[0].y, 2);
    assert_eq!(plan.placed_images[1].x, 10);
    assert_eq!(plan.placed_images[1].y, 9);
}

#[test]
fn plans_horizontal_stack_with_end_alignment() {
    let images = vec![
        solid_image(4, 8, [255, 0, 0, 255]),
        solid_image(6, 3, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::HorizontalStack,
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::End,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("horizontal plan");

    assert_eq!(plan.width, 14);
    assert_eq!(plan.height, 10);
    assert_eq!(plan.placed_images[0].x, 1);
    assert_eq!(plan.placed_images[0].y, 1);
    assert_eq!(plan.placed_images[1].x, 7);
    assert_eq!(plan.placed_images[1].y, 6);
}

#[test]
fn fit_to_same_width_resizes_height_in_layout() {
    let images = vec![
        solid_image(4, 2, [255, 0, 0, 255]),
        solid_image(2, 6, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameWidth,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("same-width plan");

    assert_eq!(plan.width, 4);
    assert_eq!(plan.height, 14);
    assert_eq!(plan.placed_images[0].width, 4);
    assert_eq!(plan.placed_images[0].height, 2);
    assert_eq!(plan.placed_images[1].width, 4);
    assert_eq!(plan.placed_images[1].height, 12);
}

#[test]
fn fit_to_same_height_resizes_width_in_layout() {
    let images = vec![
        solid_image(10, 10, [255, 0, 0, 255]),
        solid_image(6, 3, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::HorizontalStack,
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::FitToSameHeight,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("same-height plan");

    assert_eq!(plan.width, 34);
    assert_eq!(plan.height, 12);
    assert_eq!(plan.placed_images[0].width, 10);
    assert_eq!(plan.placed_images[1].width, 20);
    assert_eq!(plan.placed_images[1].height, 10);
}

#[test]
fn before_after_equalizes_pane_heights() {
    let images = vec![
        solid_image(2, 5, [255, 0, 0, 255]),
        solid_image(3, 3, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 4,
        padding: Insets::all(2),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("before-after plan");

    assert_eq!(plan.width, 15);
    assert_eq!(plan.height, 9);
    assert_eq!(plan.placed_images[0].x, 2);
    assert_eq!(plan.placed_images[0].y, 2);
    assert_eq!(
        (plan.placed_images[0].width, plan.placed_images[0].height),
        (2, 5)
    );
    assert_eq!(plan.placed_images[1].x, 8);
    assert_eq!(plan.placed_images[1].y, 2);
    assert_eq!(
        (plan.placed_images[1].width, plan.placed_images[1].height),
        (5, 5)
    );
}

#[test]
fn before_after_requires_exactly_two_images() {
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        ..ComposeOptions::default()
    };

    let three = vec![
        solid_image(1, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
        solid_image(1, 1, [0, 0, 255, 255]),
    ];
    let error = plan_layout(&three, &options).unwrap_err();
    assert!(matches!(error, ComposeError::BeforeAfterRequiresTwo));

    let one = vec![solid_image(1, 1, [255, 0, 0, 255])];
    let error = plan_layout(&one, &options).unwrap_err();
    assert!(matches!(error, ComposeError::NotEnoughImages));
}

#[test]
fn before_after_scales_shorter_pane_to_taller_height() {
    let images = vec![
        solid_image(4, 8, [255, 0, 0, 255]),
        solid_image(6, 4, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("before-after plan");

    assert_eq!(plan.width, 20);
    assert_eq!(plan.height, 10);
    assert_eq!(plan.placed_images[0].height, 8);
    assert_eq!(plan.placed_images[1].height, 8);
    assert_eq!(plan.placed_images[1].width, 12);

    let placements: Vec<(u32, u32, u32, u32)> = plan
        .placed_images
        .iter()
        .map(|placed| (placed.x, placed.y, placed.width, placed.height))
        .collect();

    assert_eq!(placements, vec![(1, 1, 4, 8), (7, 1, 12, 8)]);
}

#[test]
fn before_after_ignores_resize_mode() {
    let images = vec![
        solid_image(4, 8, [255, 0, 0, 255]),
        solid_image(6, 4, [0, 255, 0, 255]),
    ];
    let base = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };
    let resized = ComposeOptions {
        resize_mode: ResizeMode::FitToSameWidth,
        pane_style: PaneStyle::default(),
        divider: None,
        ..base.clone()
    };

    let plain_plan = plan_layout(&images, &base).expect("before-after plan");
    let resized_plan = plan_layout(&images, &resized).expect("before-after resized plan");

    assert_eq!(resized_plan, plain_plan);
}

#[test]
fn grid_reports_zero_column_error() {
    let images = vec![
        solid_image(1, 1, [255, 0, 0, 255]),
        solid_image(1, 1, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 0 },
        ..ComposeOptions::default()
    };

    let error = plan_layout(&images, &options).unwrap_err();

    assert!(matches!(error, ComposeError::InvalidGridColumns));
}

#[test]
fn plans_grid_row_major_with_track_sizes() {
    let images = vec![
        solid_image(10, 4, [255, 0, 0, 255]),
        solid_image(6, 8, [0, 255, 0, 255]),
        solid_image(4, 10, [0, 0, 255, 255]),
        solid_image(8, 6, [255, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 2 },
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("grid plan");

    assert_eq!(plan.width, 22);
    assert_eq!(plan.height, 22);
    assert_eq!(plan.placed_images.len(), 4);

    let placements: Vec<(u32, u32, u32, u32)> = plan
        .placed_images
        .iter()
        .map(|placed| (placed.x, placed.y, placed.width, placed.height))
        .collect();

    assert_eq!(
        placements,
        vec![(1, 1, 10, 4), (13, 1, 6, 8), (1, 11, 4, 10), (13, 11, 8, 6),]
    );
}

#[test]
fn grid_with_more_columns_than_images_uses_one_row() {
    let images = vec![
        solid_image(4, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
        solid_image(6, 2, [0, 0, 255, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 5 },
        spacing: 3,
        padding: Insets::all(2),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("single-row grid plan");

    let tracks = 4 + 2 + 6;
    let gaps = 2 * options.spacing;
    let padding = options.padding.left + options.padding.right;

    assert_eq!(plan.width, tracks + gaps + padding);
    assert_eq!(
        plan.height,
        2 + options.padding.top + options.padding.bottom
    );
    assert!(plan
        .placed_images
        .iter()
        .all(|placed| placed.y == options.padding.top));
}

#[test]
fn grid_alignment_applies_within_column() {
    // The second column's track is 4 wide, so the 2-wide image at index 1 has slack.
    let images = vec![
        solid_image(4, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
        solid_image(4, 2, [0, 0, 255, 255]),
        solid_image(4, 2, [255, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 2 },
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };

    let plan = plan_layout(&images, &options).expect("centered grid plan");

    assert_eq!(plan.placed_images[0].x, options.padding.left);
    assert_eq!(
        plan.placed_images[1].x,
        options.padding.left + 4 + options.spacing + 1
    );

    let plan = plan_layout(
        &images,
        &ComposeOptions {
            alignment: Alignment::End,
            ..options
        },
    )
    .expect("end-aligned grid plan");

    assert_eq!(
        plan.placed_images[1].x,
        options.padding.left + 4 + options.spacing + 2
    );
}

#[test]
fn single_row_grid_matches_horizontal_flow() {
    let images = vec![
        solid_image(4, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
    ];
    let grid_options = ComposeOptions {
        layout: LayoutKind::Grid { columns: 2 },
        spacing: 2,
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
    };
    let horizontal_options = ComposeOptions {
        layout: LayoutKind::HorizontalStack,
        ..grid_options
    };

    let grid_plan = plan_layout(&images, &grid_options).expect("grid plan");
    let horizontal_plan = plan_layout(&images, &horizontal_options).expect("horizontal plan");

    assert_eq!(grid_plan, horizontal_plan);
}

#[test]
fn divider_requires_before_after() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        divider: Some(Divider {
            width: 4,
            color: RgbaColor::WHITE,
        }),
        ..ComposeOptions::default()
    };

    let error = plan_layout(&images, &options).unwrap_err();

    assert!(matches!(error, ComposeError::DividerRequiresBeforeAfter));
}

#[test]
fn divider_wider_than_gap_is_rejected() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 2,
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 4,
            color: RgbaColor::WHITE,
        }),
    };

    let error = plan_layout(&images, &options).unwrap_err();

    assert!(matches!(
        error,
        ComposeError::DividerTooWide { width: 4, gap: 2 }
    ));
}

#[test]
fn divider_equal_to_gap_is_accepted() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 2,
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 2,
            color: RgbaColor::WHITE,
        }),
    };

    let plan = plan_layout(&images, &options).expect("divider fitting the gap");

    assert_eq!(plan.width, 10);
    assert_eq!(plan.height, 4);
    assert_eq!(plan.placed_images[0].x, 0);
    assert_eq!(plan.placed_images[1].x, 6);
    assert_eq!(
        plan.placed_images[1].x - (plan.placed_images[0].x + plan.placed_images[0].width),
        2
    );
}

#[test]
fn zero_width_divider_is_allowed_without_spacing() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let before_after = ComposeOptions {
        layout: LayoutKind::BeforeAfter,
        spacing: 0,
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 0,
            color: RgbaColor::WHITE,
        }),
    };

    let plan = plan_layout(&images, &before_after).expect("zero-width divider");

    assert_eq!(plan.width, 8);
    assert_eq!(plan.placed_images[1].x, 4);

    let no_divider = ComposeOptions {
        divider: None,
        ..before_after
    };
    assert_eq!(plan_layout(&images, &no_divider).expect("no divider"), plan);

    // The layout check keys on `Some(_)` alone, so a zero-width divider still errors elsewhere.
    let vertical = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        ..before_after
    };
    assert!(matches!(
        plan_layout(&images, &vertical).unwrap_err(),
        ComposeError::DividerRequiresBeforeAfter
    ));
}
