//! Provides facilities to implement Protocol Bindings.
//!
//! Note: these APIs should be considered unstable and subject to changes.

#[cfg(all(
    feature = "axum",
    any(
        feature = "http-binding",
        feature = "actix",
        feature = "reqwest",
        feature = "warp",
        feature = "poem"
    )
))]
compile_error!("feature `axum` cannot be used with features `http-binding`, `actix`, `reqwest`, `warp`, or `poem`");

mod deserializer;
mod encoding;
mod error;
mod serializer;
mod types;

pub use deserializer::*;
pub use encoding::*;
pub use error::*;
pub use serializer::*;
pub use types::MessageAttributeValue;
