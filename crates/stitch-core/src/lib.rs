pub mod compose;
pub mod error;
pub mod image_io;
pub mod layout;
pub mod options;
pub mod render;
pub mod types;

pub use compose::{compose, plan_layout};
pub use error::ComposeError;
pub use image_io::{load_image_from_path, save_png};
pub use options::ComposeOptions;
pub use types::{
    Alignment, CanvasPlan, ComposedImage, InputImage, Insets, LayoutKind, PlacedImage, ResizeMode,
    RgbaColor,
};
