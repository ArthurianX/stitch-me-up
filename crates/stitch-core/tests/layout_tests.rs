mod helpers;

use helpers::solid_image;
use stitch_core::{
    plan_layout, Alignment, CellSize, ComposeError, ComposeOptions, Divider, Insets, LayoutKind,
    PaneStyle, ResizeMode, RgbaColor, SourceCrop, Spacing,
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
        cell_size: None,
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(3),
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
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::HorizontalStack,
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::End,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::FitToSameWidth,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::HorizontalStack,
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::FitToSameHeight,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(4),
        padding: Insets::all(2),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };
    let resized = ComposeOptions {
        cell_size: None,
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
        cell_size: None,
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
        cell_size: None,
        layout: LayoutKind::Grid { columns: 2 },
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::Grid { columns: 5 },
        spacing: Spacing::all(3),
        padding: Insets::all(2),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("single-row grid plan");

    let tracks = 4 + 2 + 6;
    let gaps = 2 * options.spacing.x;
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
        cell_size: None,
        layout: LayoutKind::Grid { columns: 2 },
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("centered grid plan");

    assert_eq!(plan.placed_images[0].x, options.padding.left);
    assert_eq!(
        plan.placed_images[1].x,
        options.padding.left + 4 + options.spacing.x + 1
    );

    let plan = plan_layout(
        &images,
        &ComposeOptions {
            cell_size: None,
            alignment: Alignment::End,
            ..options
        },
    )
    .expect("end-aligned grid plan");

    assert_eq!(
        plan.placed_images[1].x,
        options.padding.left + 4 + options.spacing.x + 2
    );
}

#[test]
fn single_row_grid_matches_horizontal_flow() {
    let images = vec![
        solid_image(4, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
    ];
    let grid_options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::Grid { columns: 2 },
        spacing: Spacing::all(2),
        padding: Insets::all(1),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };
    let horizontal_options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::HorizontalStack,
        ..grid_options.clone()
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
        cell_size: None,
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(2),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 4,
            color: RgbaColor::WHITE,
        }),
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(2),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 2,
            color: RgbaColor::WHITE,
        }),
        column_weights: Vec::new(),
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
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing::all(0),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: Some(Divider {
            width: 0,
            color: RgbaColor::WHITE,
        }),
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &before_after).expect("zero-width divider");

    assert_eq!(plan.width, 8);
    assert_eq!(plan.placed_images[1].x, 4);

    let no_divider = ComposeOptions {
        cell_size: None,
        divider: None,
        ..before_after.clone()
    };
    assert_eq!(plan_layout(&images, &no_divider).expect("no divider"), plan);

    // The layout check keys on `Some(_)` alone, so a zero-width divider still errors elsewhere.
    let vertical = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::VerticalStack,
        ..before_after
    };
    assert!(matches!(
        plan_layout(&images, &vertical).unwrap_err(),
        ComposeError::DividerRequiresBeforeAfter
    ));
}

#[test]
fn vertical_uses_only_vertical_spacing() {
    let images = vec![
        solid_image(2, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::VerticalStack,
        spacing: Spacing { x: 50, y: 3 },
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("vertical plan");

    assert_eq!(plan.width, 2);
    assert_eq!(plan.height, 7);
    assert_eq!(plan.placed_images[1].y, 5);
}

#[test]
fn horizontal_uses_only_horizontal_spacing() {
    let images = vec![
        solid_image(2, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::HorizontalStack,
        spacing: Spacing { x: 3, y: 50 },
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("horizontal plan");

    assert_eq!(plan.width, 7);
    assert_eq!(plan.height, 2);
    assert_eq!(plan.placed_images[1].x, 5);
}

#[test]
fn grid_uses_per_axis_spacing() {
    let images = (0..4)
        .map(|index| solid_image(2, 2, [index as u8 * 40, 60, 120, 255]))
        .collect::<Vec<_>>();
    let options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::Grid { columns: 2 },
        spacing: Spacing { x: 5, y: 7 },
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("grid plan");

    assert_eq!(plan.width, 9);
    assert_eq!(plan.height, 11);
    let positions: Vec<(u32, u32)> = plan
        .placed_images
        .iter()
        .map(|placed| (placed.x, placed.y))
        .collect();
    assert_eq!(positions, vec![(0, 0), (7, 0), (0, 9), (7, 9)]);
}

#[test]
fn before_after_uses_horizontal_spacing() {
    let images = vec![
        solid_image(2, 2, [255, 0, 0, 255]),
        solid_image(2, 2, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::BeforeAfter,
        spacing: Spacing { x: 6, y: 40 },
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    };

    let plan = plan_layout(&images, &options).expect("before-after plan");

    assert_eq!(plan.width, 10);
    assert_eq!(plan.height, 2);
    assert_eq!(plan.placed_images[1].x, 8);
}

fn grid_weight_options(columns: u32, weights: Vec<u32>) -> ComposeOptions {
    ComposeOptions {
        cell_size: None,
        layout: LayoutKind::Grid { columns },
        spacing: Spacing::default(),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Center,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: weights,
    }
}

fn weighted_images() -> Vec<stitch_core::InputImage> {
    vec![
        solid_image(40, 30, [255, 0, 0, 255]),
        solid_image(20, 60, [0, 255, 0, 255]),
        solid_image(30, 30, [0, 0, 255, 255]),
    ]
}

#[test]
fn grid_weights_expand_columns_proportionally() {
    let plan = plan_layout(&weighted_images(), &grid_weight_options(2, vec![1, 2]))
        .expect("weighted grid plan");

    // natural [40, 20]; unit = max(40/1, ceil(20/2)) = 40; tracks [40, 80].
    assert_eq!(plan.width, 120);
    assert_eq!(plan.height, 90);
    assert_eq!(plan.placed_images[0].x, 0);
    assert_eq!(plan.placed_images[0].y, 0);
    assert_eq!(plan.placed_images[0].width, 40);
    assert_eq!(plan.placed_images[1].x, 70);
    assert_eq!(plan.placed_images[1].y, 0);
    assert_eq!(plan.placed_images[1].width, 20);
    assert_eq!(plan.placed_images[2].x, 5);
    assert_eq!(plan.placed_images[2].y, 60);
    assert_eq!(plan.placed_images[2].width, 30);
}

#[test]
fn grid_equal_weights_make_uniform_tracks() {
    let plan = plan_layout(&weighted_images(), &grid_weight_options(2, vec![1, 1]))
        .expect("uniform weighted grid plan");

    assert_eq!(plan.width, 80);
    assert_eq!(plan.height, 90);
    assert_eq!(plan.placed_images[1].x, 50);
}

#[test]
fn grid_weights_never_shrink_below_content() {
    let images = vec![
        solid_image(40, 10, [255, 0, 0, 255]),
        solid_image(10, 10, [0, 255, 0, 255]),
    ];
    let options = ComposeOptions {
        cell_size: None,
        alignment: Alignment::Start,
        ..grid_weight_options(2, vec![2, 1])
    };

    let plan = plan_layout(&images, &options).expect("weighted grid plan");

    // natural [40, 10]; unit = max(20, 10) = 20; tracks [40, 20] — never below content.
    assert_eq!(plan.width, 60);
    assert_eq!(plan.height, 10);
    assert_eq!(plan.placed_images[1].x, 40);
    assert_eq!(plan.placed_images[1].width, 10);
}

#[test]
fn grid_weights_must_match_columns() {
    let error = plan_layout(&weighted_images(), &grid_weight_options(2, vec![1])).unwrap_err();

    assert!(matches!(
        error,
        ComposeError::ColumnWeightCountMismatch {
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn grid_weights_reject_zero() {
    let error = plan_layout(&weighted_images(), &grid_weight_options(2, vec![0, 1])).unwrap_err();

    assert!(matches!(error, ComposeError::ZeroColumnWeight { index: 0 }));
}

#[test]
fn column_weights_require_grid_layout() {
    let options = ComposeOptions {
        cell_size: None,
        layout: LayoutKind::VerticalStack,
        ..grid_weight_options(2, vec![1, 1])
    };

    let error = plan_layout(&weighted_images(), &options).unwrap_err();

    assert!(matches!(error, ComposeError::ColumnWeightsRequireGrid));
}

#[test]
fn empty_column_weights_keep_auto_sizing() {
    let plan = plan_layout(&weighted_images(), &grid_weight_options(2, Vec::new()))
        .expect("auto-sized grid plan");

    // Same fixture as the weighted tests: columns stay at their natural [40, 20]
    // widths, rows at [60, 30] — matching the pre-weight grid fixtures above.
    assert_eq!(plan.width, 60);
    assert_eq!(plan.height, 90);
    assert_eq!(plan.placed_images[0].x, 0);
    assert_eq!(plan.placed_images[0].y, 0);
    assert_eq!(plan.placed_images[1].x, 40);
    assert_eq!(plan.placed_images[1].y, 0);
    assert_eq!(plan.placed_images[2].x, 5);
    assert_eq!(plan.placed_images[2].y, 60);
}

fn cell_options(columns: u32, width: u32, height: u32) -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::Grid { columns },
        spacing: Spacing::all(8),
        padding: Insets::default(),
        alignment: Alignment::Center,
        cell_size: Some(CellSize { width, height }),
        ..ComposeOptions::default()
    }
}

#[test]
fn plans_uniform_grid_cells_for_six_images() {
    let images = (0..6u32)
        .map(|i| solid_image(40 + i * 10, 30 + i * 5, [i as u8, 0, 0, 255]))
        .collect::<Vec<_>>();
    let plan = plan_layout(&images, &cell_options(3, 320, 320)).expect("cell grid");

    assert_eq!((plan.width, plan.height), (976, 648));
    assert_eq!(plan.placed_images.len(), 6);
    for (index, placed) in plan.placed_images.iter().enumerate() {
        assert_eq!((placed.width, placed.height), (320, 320));
        assert_eq!(placed.x, (index as u32 % 3) * 328);
        assert_eq!(placed.y, (index as u32 / 3) * 328);
    }
}

#[test]
fn cell_mode_keeps_padding_spacing_and_partial_rows() {
    let images = (0..4u32)
        .map(|_| solid_image(8, 8, [0, 255, 0, 255]))
        .collect::<Vec<_>>();
    let options = ComposeOptions {
        spacing: Spacing { x: 2, y: 3 },
        padding: Insets::all(5),
        ..cell_options(3, 10, 20)
    };
    let plan = plan_layout(&images, &options).expect("cell grid");

    assert_eq!((plan.width, plan.height), (44, 53));
    let last = &plan.placed_images[3];
    assert_eq!((last.x, last.y), (5, 28));
}

#[test]
fn cell_mode_computes_centered_source_crops() {
    let wide = vec![solid_image(200, 50, [0; 4]), solid_image(200, 50, [0; 4])];
    let plan = plan_layout(&wide, &cell_options(2, 100, 100)).expect("wide");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop {
            x: 75,
            y: 0,
            width: 50,
            height: 50
        })
    );

    let tall = vec![solid_image(50, 200, [0; 4]), solid_image(50, 200, [0; 4])];
    let plan = plan_layout(&tall, &cell_options(2, 100, 100)).expect("tall");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop {
            x: 0,
            y: 75,
            width: 50,
            height: 50
        })
    );

    let same = vec![solid_image(200, 100, [0; 4]), solid_image(200, 100, [0; 4])];
    let plan = plan_layout(&same, &cell_options(2, 100, 50)).expect("same aspect");
    assert_eq!(plan.placed_images[0].crop, None);

    let exact = vec![solid_image(100, 100, [0; 4]), solid_image(100, 100, [0; 4])];
    let plan = plan_layout(&exact, &cell_options(2, 100, 100)).expect("exact");
    assert_eq!(plan.placed_images[0].crop, None);

    // Odd excess: 7 wide → 4 wide leaves 3 columns; floor offset 1 drops one from
    // the left and two from the right (the extra pixel goes to the right edge).
    let odd = vec![solid_image(7, 4, [0; 4]), solid_image(7, 4, [0; 4])];
    let plan = plan_layout(&odd, &cell_options(2, 3, 3)).expect("odd");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop {
            x: 1,
            y: 0,
            width: 4,
            height: 4
        })
    );
}

#[test]
fn cell_mode_clamps_columns_and_rejects_zero() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];

    let plan = plan_layout(&images, &cell_options(5, 10, 10)).expect("clamped");
    assert_eq!(plan.placed_images.len(), 2);
    assert_eq!((plan.width, plan.height), (28, 10));
    assert_eq!(plan.placed_images[1].x, 18);
    assert_eq!(plan.placed_images[1].y, 0);

    assert!(matches!(
        plan_layout(&images, &cell_options(0, 10, 10)).unwrap_err(),
        ComposeError::InvalidGridColumns
    ));
}

#[test]
fn rejects_cell_size_outside_grid() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        cell_size: Some(CellSize {
            width: 4,
            height: 4,
        }),
        ..ComposeOptions::default()
    };
    assert!(matches!(
        plan_layout(&images, &options).unwrap_err(),
        ComposeError::CellSizeRequiresGrid
    ));
}

#[test]
fn rejects_zero_cell_dimensions() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];
    let options = cell_options(2, 0, 10);
    assert!(matches!(
        plan_layout(&images, &options).unwrap_err(),
        ComposeError::InvalidCellSize {
            width: 0,
            height: 10
        }
    ));
}

#[test]
fn rejects_cell_size_conflicts() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];

    let weights = ComposeOptions {
        column_weights: vec![1, 1],
        ..cell_options(2, 10, 10)
    };
    assert!(matches!(
        plan_layout(&images, &weights).unwrap_err(),
        ComposeError::CellSizeConflictsWithColumnWeights
    ));

    let resize = ComposeOptions {
        resize_mode: ResizeMode::FitToSameWidth,
        ..cell_options(2, 10, 10)
    };
    assert!(matches!(
        plan_layout(&images, &resize).unwrap_err(),
        ComposeError::CellSizeConflictsWithResizeMode
    ));
}
