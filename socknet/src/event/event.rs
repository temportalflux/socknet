use crate::packet::Packet;
use std::net::SocketAddr;

#[derive(Clone)]
pub enum Event {
	Connected(SocketAddr),
	TimedOut(SocketAddr),
	Disconnected(SocketAddr),
	Packet(Packet),
	Stop,
}

impl std::fmt::Debug for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Connected(address) => write!(f, "Connected({})", address),
			Self::TimedOut(address) => write!(f, "TimedOut({})", address),
			Self::Disconnected(address) => write!(f, "Disconnected({})", address),
			Self::Packet(packet) => write!(f, "Packet({:?})", packet),
			Self::Stop => write!(f, "Stop"),
		}
	}
}

impl From<laminar::SocketEvent> for Event {
	fn from(event: laminar::SocketEvent) -> Self {
		use laminar::SocketEvent::*;
		match event {
			Connect(address) => Self::Connected(address),
			Timeout(address) => Self::TimedOut(address),
			Disconnect(address) => Self::Disconnected(address),
			Packet(packet) => Self::Packet(packet.into()),
		}
	}
}

impl Event {
	pub fn address(&self) -> Option<SocketAddr> {
		return match self {
			Self::Connected(addr) => Some(*addr),
			Self::TimedOut(addr) => Some(*addr),
			Self::Disconnected(addr) => Some(*addr),
			Self::Packet(packet) => Some(*packet.address()),
			Self::Stop => None,
		};
	}
}
