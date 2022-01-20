use crate::{backend, channel};
use std::net::SocketAddr;

pub struct Factory();
impl super::Factory for Factory {
	fn build(
		address: SocketAddr,
		config: backend::Config,
	) -> anyhow::Result<Box<dyn super::ISocket + Send>> {
		Ok(Box::new(Socket(laminar::Socket::bind_with_config(
			address, config,
		)?)))
	}
}

pub struct Socket(laminar::Socket);
impl super::ISocket for Socket {
	fn get_packet_sender(&self) -> channel::Sender<backend::Packet> {
		self.0.get_packet_sender()
	}
	fn get_event_receiver(&self) -> channel::Receiver<backend::Event> {
		self.0.get_event_receiver()
	}
	fn manual_poll(&mut self, time: std::time::Instant) {
		self.0.manual_poll(time);
	}
	fn kick(&mut self, address: &SocketAddr) {
		self.0.kick(address);
	}
}
