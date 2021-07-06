//! Provides protocol binding implementations for [`Event`].

#[cfg(feature = "actix-binding")]
pub mod actix;
#[cfg(feature = "rdkafka-binding")]
pub mod rdkafka;
#[cfg(feature = "reqwest-binding")]
pub mod reqwest;
#[cfg(feature = "warp-binding")]
pub mod warp;
