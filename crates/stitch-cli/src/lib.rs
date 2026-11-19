mod color;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use stitch_core::{
    compose, load_image_from_path, save_png, Alignment, ComposeOptions, Divider, Frame, Insets,
    LayoutKind, PaneStyle, ResizeMode, RgbaColor, Spacing,
};

pub use color::parse_rgba_color;

#[derive(Debug, Parser)]
#[command(
    name = "stitch-me-up",
    version,
    about = "🩹 Combine multiple images into a single canvas",
    long_about = "🩹 Stitch Me Up combines local PNG or JPEG images into one composed PNG using predictable layout rules."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Combine images into a single canvas")]
    Combine(CombineArgs),
}

#[derive(Debug, Args)]
pub struct CombineArgs {
    #[arg(
        value_name = "INPUT",
        required = true,
        num_args = 2..,
        help = "Input PNG or JPEG files"
    )]
    pub inputs: Vec<PathBuf>,

    #[arg(short, long, value_name = "FILE", help = "Output PNG file")]
    pub output: PathBuf,

    #[arg(long, value_enum, default_value_t = LayoutArg::Vertical)]
    pub layout: LayoutArg,

    #[arg(
        long,
        default_value_t = 0,
        help = "Space between images in pixels, overridden per axis by --spacing-x/--spacing-y"
    )]
    pub spacing: u32,

    #[arg(
        long,
        help = "Horizontal gap between images in pixels, overriding --spacing"
    )]
    pub spacing_x: Option<u32>,

    #[arg(
        long,
        help = "Vertical gap between images in pixels, overriding --spacing"
    )]
    pub spacing_y: Option<u32>,

    #[arg(
        long,
        default_value_t = 0,
        help = "Uniform padding around the canvas, overridden per side by the --padding-* flags"
    )]
    pub padding: u32,

    #[arg(long, help = "Padding above the canvas, overriding --padding")]
    pub padding_top: Option<u32>,

    #[arg(
        long,
        help = "Padding to the right of the canvas, overriding --padding"
    )]
    pub padding_right: Option<u32>,

    #[arg(long, help = "Padding below the canvas, overriding --padding")]
    pub padding_bottom: Option<u32>,

    #[arg(long, help = "Padding to the left of the canvas, overriding --padding")]
    pub padding_left: Option<u32>,

    #[arg(
        long,
        default_value = "#ffffff",
        value_parser = parse_rgba_color,
        help = "Background color as #RRGGBB or #RRGGBBAA"
    )]
    pub background: RgbaColor,

    #[arg(long, value_enum, default_value_t = AlignmentArg::Center)]
    pub alignment: AlignmentArg,

    #[arg(long, value_enum, default_value_t = ResizeModeArg::None)]
    pub resize_mode: ResizeModeArg,

    #[arg(
        long,
        default_value_t = 2,
        help = "Grid column count when layout is grid"
    )]
    pub columns: u32,

    #[arg(
        long,
        default_value_t = 0,
        help = "Frame width in pixels drawn inside each image"
    )]
    pub frame_width: u32,

    #[arg(
        long,
        default_value = "#000000",
        value_parser = parse_rgba_color,
        help = "Frame color as #RRGGBB or #RRGGBBAA"
    )]
    pub frame_color: RgbaColor,

    #[arg(
        long,
        default_value_t = 0,
        help = "Corner radius in pixels applied to each image"
    )]
    pub corner_radius: u32,

    #[arg(
        long,
        default_value_t = 0,
        help = "Divider width in pixels between before-after panes"
    )]
    pub divider_width: u32,

    #[arg(
        long,
        default_value = "#000000",
        value_parser = parse_rgba_color,
        help = "Divider color as #RRGGBB or #RRGGBBAA"
    )]
    pub divider_color: RgbaColor,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LayoutArg {
    Vertical,
    Horizontal,
    Grid,
    BeforeAfter,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AlignmentArg {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResizeModeArg {
    None,
    FitToSameWidth,
    FitToSameHeight,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    run_cli(cli)
}

pub fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Combine(args) => combine(args),
    }
}

pub fn combine(args: CombineArgs) -> Result<()> {
    if matches!(args.layout, LayoutArg::BeforeAfter) && args.inputs.len() != 2 {
        bail!("before-after layout requires exactly 2 input images");
    }

    if matches!(args.layout, LayoutArg::Grid) && args.columns == 0 {
        bail!("grid layout requires at least 1 column");
    }

    let images = args
        .inputs
        .iter()
        .map(|path| {
            load_image_from_path(path)
                .with_context(|| format!("failed to load input image {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;

    let options = ComposeOptions {
        layout: layout_kind(args.layout, args.columns),
        spacing: spacing(args.spacing, args.spacing_x, args.spacing_y),
        padding: insets(
            args.padding,
            args.padding_top,
            args.padding_right,
            args.padding_bottom,
            args.padding_left,
        ),
        background: args.background,
        alignment: alignment(args.alignment),
        resize_mode: resize_mode(args.resize_mode),
        pane_style: pane_style(args.frame_width, args.frame_color, args.corner_radius),
        divider: divider(args.divider_width, args.divider_color),
    };

    let composed = compose(&images, &options).context("failed to compose images")?;
    save_png(&args.output, &composed)
        .with_context(|| format!("failed to write PNG output {}", args.output.display()))?;

    println!("Wrote {}", args.output.display());
    Ok(())
}

pub fn layout_kind(layout: LayoutArg, columns: u32) -> LayoutKind {
    match layout {
        LayoutArg::Vertical => LayoutKind::VerticalStack,
        LayoutArg::Horizontal => LayoutKind::HorizontalStack,
        LayoutArg::Grid => LayoutKind::Grid { columns },
        LayoutArg::BeforeAfter => LayoutKind::BeforeAfter,
    }
}

pub fn alignment(alignment: AlignmentArg) -> Alignment {
    match alignment {
        AlignmentArg::Start => Alignment::Start,
        AlignmentArg::Center => Alignment::Center,
        AlignmentArg::End => Alignment::End,
    }
}

pub fn resize_mode(mode: ResizeModeArg) -> ResizeMode {
    match mode {
        ResizeModeArg::None => ResizeMode::None,
        ResizeModeArg::FitToSameWidth => ResizeMode::FitToSameWidth,
        ResizeModeArg::FitToSameHeight => ResizeMode::FitToSameHeight,
    }
}

pub fn spacing(uniform: u32, x: Option<u32>, y: Option<u32>) -> Spacing {
    Spacing {
        x: x.unwrap_or(uniform),
        y: y.unwrap_or(uniform),
    }
}

pub fn insets(
    uniform: u32,
    top: Option<u32>,
    right: Option<u32>,
    bottom: Option<u32>,
    left: Option<u32>,
) -> Insets {
    Insets {
        top: top.unwrap_or(uniform),
        right: right.unwrap_or(uniform),
        bottom: bottom.unwrap_or(uniform),
        left: left.unwrap_or(uniform),
    }
}

pub fn pane_style(frame_width: u32, frame_color: RgbaColor, corner_radius: u32) -> PaneStyle {
    PaneStyle {
        frame: (frame_width > 0).then_some(Frame {
            width: frame_width,
            color: frame_color,
        }),
        corner_radius,
    }
}

pub fn divider(width: u32, color: RgbaColor) -> Option<Divider> {
    (width > 0).then_some(Divider { width, color })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn sample_args() -> Vec<&'static str> {
        vec![
            "stitch-me-up",
            "combine",
            "--output",
            "out.png",
            "a.png",
            "b.png",
        ]
    }

    #[test]
    fn parses_minimal_combine_command() {
        let cli = Cli::parse_from(sample_args());

        match cli.command {
            Commands::Combine(args) => {
                assert_eq!(args.output, PathBuf::from("out.png"));
                assert_eq!(
                    args.inputs,
                    vec![PathBuf::from("a.png"), PathBuf::from("b.png")]
                );
                assert!(matches!(args.layout, LayoutArg::Vertical));
                assert!(matches!(args.alignment, AlignmentArg::Center));
                assert!(matches!(args.resize_mode, ResizeModeArg::None));
                assert_eq!(args.spacing, 0);
                assert_eq!(args.padding, 0);
                assert_eq!(
                    args.background,
                    RgbaColor {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    }
                );
            }
        }
    }

    #[test]
    fn parses_explicit_layout_and_style_flags() {
        let cli = Cli::parse_from([
            "stitch-me-up",
            "combine",
            "--layout",
            "horizontal",
            "--spacing",
            "12",
            "--padding",
            "8",
            "--background",
            "#11223344",
            "--alignment",
            "end",
            "--resize-mode",
            "fit-to-same-height",
            "--output",
            "joined.png",
            "a.png",
            "b.png",
        ]);

        match cli.command {
            Commands::Combine(args) => {
                assert!(matches!(args.layout, LayoutArg::Horizontal));
                assert_eq!(args.spacing, 12);
                assert_eq!(args.padding, 8);
                assert_eq!(
                    args.background,
                    RgbaColor {
                        r: 0x11,
                        g: 0x22,
                        b: 0x33,
                        a: 0x44,
                    }
                );
                assert!(matches!(args.alignment, AlignmentArg::End));
                assert!(matches!(args.resize_mode, ResizeModeArg::FitToSameHeight));
                assert_eq!(args.output, PathBuf::from("joined.png"));
            }
        }
    }

    #[test]
    fn per_side_padding_overrides_uniform_padding() {
        let cli = Cli::parse_from([
            "stitch-me-up",
            "combine",
            "--padding",
            "4",
            "--padding-left",
            "20",
            "--padding-top",
            "0",
            "--output",
            "out.png",
            "a.png",
            "b.png",
        ]);

        match cli.command {
            Commands::Combine(args) => {
                assert_eq!(args.padding, 4);
                assert_eq!(args.padding_top, Some(0));
                assert_eq!(args.padding_right, None);
                assert_eq!(args.padding_bottom, None);
                assert_eq!(args.padding_left, Some(20));

                let padding = insets(
                    args.padding,
                    args.padding_top,
                    args.padding_right,
                    args.padding_bottom,
                    args.padding_left,
                );

                assert_eq!(
                    padding,
                    Insets {
                        top: 0,
                        right: 4,
                        bottom: 4,
                        left: 20,
                    }
                );
            }
        }
    }

    #[test]
    fn per_side_padding_defaults_to_uniform_padding() {
        let cli = Cli::parse_from([
            "stitch-me-up",
            "combine",
            "--padding",
            "7",
            "--output",
            "out.png",
            "a.png",
            "b.png",
        ]);

        match cli.command {
            Commands::Combine(args) => {
                assert_eq!(args.padding_top, None);
                assert_eq!(args.padding_right, None);
                assert_eq!(args.padding_bottom, None);
                assert_eq!(args.padding_left, None);

                let padding = insets(
                    args.padding,
                    args.padding_top,
                    args.padding_right,
                    args.padding_bottom,
                    args.padding_left,
                );

                assert_eq!(padding, Insets::all(7));
            }
        }
    }

    #[test]
    fn per_axis_spacing_overrides_uniform_spacing() {
        let cli = Cli::parse_from([
            "stitch-me-up",
            "combine",
            "--spacing",
            "4",
            "--spacing-x",
            "20",
            "--output",
            "out.png",
            "a.png",
            "b.png",
        ]);

        match cli.command {
            Commands::Combine(args) => {
                assert_eq!(args.spacing, 4);
                assert_eq!(args.spacing_x, Some(20));
                assert_eq!(args.spacing_y, None);

                assert_eq!(
                    spacing(args.spacing, args.spacing_x, args.spacing_y),
                    Spacing { x: 20, y: 4 }
                );
            }
        }
    }

    #[test]
    fn per_axis_spacing_defaults_to_uniform_spacing() {
        let cli = Cli::parse_from([
            "stitch-me-up",
            "combine",
            "--spacing",
            "7",
            "--output",
            "out.png",
            "a.png",
            "b.png",
        ]);

        match cli.command {
            Commands::Combine(args) => {
                assert_eq!(args.spacing_x, None);
                assert_eq!(args.spacing_y, None);

                assert_eq!(
                    spacing(args.spacing, args.spacing_x, args.spacing_y),
                    Spacing::all(7)
                );
            }
        }
    }

    #[test]
    fn rejects_before_after_with_more_than_two_inputs() {
        let result = combine(CombineArgs {
            inputs: vec!["a.png".into(), "b.png".into(), "c.png".into()],
            output: "out.png".into(),
            layout: LayoutArg::BeforeAfter,
            spacing: 0,
            spacing_x: None,
            spacing_y: None,
            padding: 0,
            padding_top: None,
            padding_right: None,
            padding_bottom: None,
            padding_left: None,
            background: RgbaColor::WHITE,
            alignment: AlignmentArg::Center,
            resize_mode: ResizeModeArg::None,
            columns: 2,
            frame_width: 0,
            frame_color: RgbaColor::WHITE,
            corner_radius: 0,
            divider_width: 0,
            divider_color: RgbaColor::WHITE,
        });

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "before-after layout requires exactly 2 input images"
        );
    }

    #[test]
    fn rejects_grid_with_zero_columns() {
        let result = combine(CombineArgs {
            inputs: vec!["a.png".into(), "b.png".into()],
            output: "out.png".into(),
            layout: LayoutArg::Grid,
            spacing: 0,
            spacing_x: None,
            spacing_y: None,
            padding: 0,
            padding_top: None,
            padding_right: None,
            padding_bottom: None,
            padding_left: None,
            background: RgbaColor::WHITE,
            alignment: AlignmentArg::Center,
            resize_mode: ResizeModeArg::None,
            columns: 0,
            frame_width: 0,
            frame_color: RgbaColor::WHITE,
            corner_radius: 0,
            divider_width: 0,
            divider_color: RgbaColor::WHITE,
        });

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "grid layout requires at least 1 column"
        );
    }

    #[test]
    fn maps_layout_variants_to_core_types() {
        assert_eq!(
            layout_kind(LayoutArg::Vertical, 2),
            LayoutKind::VerticalStack
        );
        assert_eq!(
            layout_kind(LayoutArg::Horizontal, 2),
            LayoutKind::HorizontalStack
        );
        assert_eq!(
            layout_kind(LayoutArg::BeforeAfter, 2),
            LayoutKind::BeforeAfter
        );
        assert_eq!(
            layout_kind(LayoutArg::Grid, 3),
            LayoutKind::Grid { columns: 3 }
        );
    }

    #[test]
    fn maps_alignment_variants_to_core_types() {
        assert_eq!(alignment(AlignmentArg::Start), Alignment::Start);
        assert_eq!(alignment(AlignmentArg::Center), Alignment::Center);
        assert_eq!(alignment(AlignmentArg::End), Alignment::End);
    }

    #[test]
    fn maps_pane_style_from_flags() {
        let plain = pane_style(0, RgbaColor::WHITE, 6);
        assert_eq!(
            plain,
            PaneStyle {
                frame: None,
                corner_radius: 6,
            }
        );

        let framed = pane_style(4, parse_rgba_color("#11223344").unwrap(), 3);
        assert_eq!(
            framed,
            PaneStyle {
                frame: Some(Frame {
                    width: 4,
                    color: RgbaColor {
                        r: 0x11,
                        g: 0x22,
                        b: 0x33,
                        a: 0x44,
                    },
                }),
                corner_radius: 3,
            }
        );
    }

    #[test]
    fn maps_divider_from_flags() {
        assert_eq!(divider(0, RgbaColor::WHITE), None);

        let stroke = divider(4, parse_rgba_color("#11223344").unwrap());
        assert_eq!(
            stroke,
            Some(Divider {
                width: 4,
                color: RgbaColor {
                    r: 0x11,
                    g: 0x22,
                    b: 0x33,
                    a: 0x44,
                },
            })
        );
    }

    #[test]
    fn maps_resize_modes_to_core_types() {
        assert_eq!(resize_mode(ResizeModeArg::None), ResizeMode::None);
        assert_eq!(
            resize_mode(ResizeModeArg::FitToSameWidth),
            ResizeMode::FitToSameWidth
        );
        assert_eq!(
            resize_mode(ResizeModeArg::FitToSameHeight),
            ResizeMode::FitToSameHeight
        );
    }
}
