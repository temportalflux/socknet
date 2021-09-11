pub type AnyError = Box<dyn std::error::Error>;
pub type VoidResult = std::result::Result<(), AnyError>;

pub static LOG: &'static str = "socknet";

#[cfg(feature = "derive")]
pub use socknet_derive::packet_kind;

pub mod event;
pub mod packet;

mod socket;
pub use socket::*;

pub mod serde {
	pub use rmp_serde::{from_read_ref, to_vec};
}

pub use crossbeam_channel as channel;
