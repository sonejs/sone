// Index-parallel loops and wide layout signatures mirror the TypeScript engine's
// algorithms line for line; renaming them into iterator form would make the two
// harder to diff. Float literals are copied verbatim from the CSS filter spec.
#![allow(
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::large_enum_variant,
    clippy::excessive_precision
)]

pub mod assets;
pub mod fonts;
pub mod image;
pub mod painter;
pub mod render;
pub mod text;

pub use assets::Assets;
pub use fonts::FontRegistry;
pub use painter::SkiaPainter;
pub use render::{Engine, RenderOptions};
pub use text::SkiaTextEngine;
