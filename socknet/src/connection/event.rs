use crate::connection::{active::Active, Connection};
use std::{net::SocketAddr, sync::Weak};

pub type Sender = async_channel::Sender<Event>;
pub type Receiver = async_channel::Receiver<Event>;

pub enum Event {
	Created(Weak<Connection>),
	Dropped(SocketAddr),
}
impl std::fmt::Debug for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match &self {
			Self::Created(connection) => write!(
				f,
				"Create({})",
				connection.upgrade().unwrap().remote_address()
			),
			Self::Dropped(address) => write!(f, "Dropped({})", address),
		}
	}
}
