use crate::{backend, channel, event, packet, AnyError};
use std::{
	net::{IpAddr, Ipv4Addr, SocketAddr},
	sync::{atomic::AtomicBool, Arc},
	thread::{self, JoinHandle},
	time::Duration,
};

pub mod default;

mod factory;
pub use factory::*;

/// The maximum size a given packet can be. If a packet is larger than this, it will be fragmented.
/// It defaults to `1450` bytes, due to the default MTU on most network devices being `1500`.
pub static FRAGMENT_SIZE: u16 = 1450;
/// Value which can specify the maximum size a packet can be in bytes.
/// This value is inclusive of fragmenting; if a packet is fragmented,
/// the total size of the fragments cannot exceed this value.
pub static MAX_PACKET_SIZE: usize = 16384;

pub(crate) fn build_thread<F, T>(name: String, f: F) -> std::io::Result<JoinHandle<T>>
where
	F: FnOnce() -> T,
	F: Send + 'static,
	T: Send + 'static,
{
	thread::Builder::new().name(name).spawn(f)
}

pub fn start(
	port: u16,
	exit_flag: &Arc<AtomicBool>,
) -> Result<(packet::Queue, event::Queue), AnyError> {
	start_as::<default::Factory>(port, exit_flag)
}

pub fn start_as<TSocketFactory>(
	port: u16,
	exit_flag: &Arc<AtomicBool>,
) -> Result<(packet::Queue, event::Queue), AnyError>
where
	TSocketFactory: Factory,
{
	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
	let name = format!("network:socket({})", address);
	let config = backend::Config {
		idle_connection_timeout: Duration::from_secs(5),
		heartbeat_interval: Some(Duration::from_millis(2 * 1000)),
		max_packet_size: MAX_PACKET_SIZE,
		fragment_size: FRAGMENT_SIZE,
		receive_buffer_max_size: FRAGMENT_SIZE as usize,
		..Default::default()
	};
	let socket = TSocketFactory::build(address, config)?;
	let (internal_sender, internal_receiver) = channel::unbounded();
	let outgoing_queue = packet::Queue::new(
		format!("{}:outgoing", name),
		socket.get_packet_sender(),
		&exit_flag,
		internal_sender,
	)?;
	let incoming_queue = event::Queue::new(
		format!("{}:incoming", name),
		socket,
		&exit_flag,
		internal_receiver,
	)?;
	Ok((outgoing_queue, incoming_queue))
}
