// Index-parallel loops and wide layout signatures mirror the TypeScript engine's
// algorithms line for line; renaming them into iterator form would make the two
// harder to diff. Float literals are copied verbatim from the CSS filter spec.
#![allow(
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::large_enum_variant,
    clippy::excessive_precision
)]

pub mod compile;
pub mod css;
pub mod draw;
pub mod dump;
pub mod error;
pub mod ir;
pub mod layout;
pub mod metadata;
pub mod pagination;
pub mod paint;
pub mod squircle;
pub mod style;
pub mod testing;
pub mod text;

pub use error::{Result, SoneError};
