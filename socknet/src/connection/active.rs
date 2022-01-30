use crate::{
	connection::Datagram,
	stream::kind::{recv, send},
	utility::PinFutureResultLifetime,
};
use std::net::SocketAddr;

mod remote;
pub use remote::*;

mod local;
pub use local::*;

pub trait Active {
	fn remote_address(&self) -> SocketAddr;
	fn peer_identity(&self) -> Option<Box<dyn std::any::Any>>;
	fn is_local(&self) -> bool;
	fn open_uni<'a>(&'a self) -> PinFutureResultLifetime<'a, send::Ongoing>;
	fn open_bi<'a>(&'a self) -> PinFutureResultLifetime<'a, (send::Ongoing, recv::Ongoing)>;
	fn send_datagram(&self, datagram: Datagram) -> anyhow::Result<()>;
	fn close(&self, code: u32, reason: &[u8]);
}
