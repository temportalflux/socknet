
pub static LOG: &'static str = "socknet";

#[cfg(feature = "derive")]
pub use socknet_derive::*;

mod config;
pub use config::*;

pub mod connection;
pub mod endpoint;
pub mod stream;
pub mod utility;
