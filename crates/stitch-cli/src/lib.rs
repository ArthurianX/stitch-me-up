mod color;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use stitch_core::{
    compose, load_image_from_path, save_png, Alignment, ComposeOptions, Insets, LayoutKind,
    ResizeMode, RgbaColor,
};

pub use color::parse_rgba_color;

#[derive(Debug, Parser)]
#[command(
    name = "stitch-me-up",
    version,
    about = "Combine multiple images into a single canvas",
    long_about = "Combine local PNG or JPEG images into one composed PNG using predictable layout rules."
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

    #[arg(long, default_value_t = 0, help = "Space between images in pixels")]
    pub spacing: u32,

    #[arg(long, default_value_t = 0, help = "Uniform padding around the canvas")]
    pub padding: u32,

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
        spacing: args.spacing,
        padding: Insets {
            top: args.padding,
            right: args.padding,
            bottom: args.padding,
            left: args.padding,
        },
        background: args.background,
        alignment: alignment(args.alignment),
        resize_mode: resize_mode(args.resize_mode),
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
    fn rejects_before_after_with_more_than_two_inputs() {
        let result = combine(CombineArgs {
            inputs: vec!["a.png".into(), "b.png".into(), "c.png".into()],
            output: "out.png".into(),
            layout: LayoutArg::BeforeAfter,
            spacing: 0,
            padding: 0,
            background: RgbaColor::WHITE,
            alignment: AlignmentArg::Center,
            resize_mode: ResizeModeArg::None,
            columns: 2,
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
            padding: 0,
            background: RgbaColor::WHITE,
            alignment: AlignmentArg::Center,
            resize_mode: ResizeModeArg::None,
            columns: 0,
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
