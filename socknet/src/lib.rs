pub use crossbeam_channel as channel;

pub static LOG: &'static str = "socknet";

#[cfg(feature = "derive")]
pub use socknet_derive::{initiator, packet_kind, responder};

// OLD
pub mod event;
pub mod packet;
pub mod socket;
pub mod backend {
	pub use laminar::{Config, Connection, ConnectionManager, Packet, SocketEvent as Event};
}
pub(crate) enum InternalMessage {
	DropConnection(std::net::SocketAddr),
}
pub mod serde {
	pub use rmp_serde::{from_read_ref, to_vec};
}

// NEW
mod config;
pub use config::*;

pub mod connection;
pub mod endpoint;
pub mod message;
pub mod stream;
pub mod utility;
