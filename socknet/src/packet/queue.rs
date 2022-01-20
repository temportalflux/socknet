use crate::{channel, packet, socket::build_thread};
use std::{
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
	thread::{self, JoinHandle},
	time::Duration,
};

pub struct Queue {
	internal_sender: channel::Sender<crate::InternalMessage>,
	sender: crossbeam_channel::Sender<packet::Packet>,
	thread_send_packets: Option<JoinHandle<()>>,
}

impl Queue {
	pub(crate) fn new(
		name: String,
		laminar_sender: crossbeam_channel::Sender<laminar::Packet>,
		exit_flag: &Arc<AtomicBool>,
		internal_sender: channel::Sender<crate::InternalMessage>,
	) -> anyhow::Result<Self> {
		let (sender, socknet_to_laminar_receiver) = crossbeam_channel::unbounded();

		let thread_exit_flag = exit_flag.clone();
		let thread_send_packets = Some(build_thread(name, move || {
			profiling::register_thread!("socknet-sender");
			Self::send_packets(
				socknet_to_laminar_receiver,
				laminar_sender,
				thread_exit_flag,
			);
		})?);

		Ok(Self {
			sender,
			thread_send_packets,
			internal_sender,
		})
	}

	fn send_packets(
		socknet_to_laminar_receiver: crossbeam_channel::Receiver<packet::Packet>,
		sender: crossbeam_channel::Sender<laminar::Packet>,
		exit_flag: Arc<AtomicBool>,
	) {
		use crossbeam_channel::{TryRecvError, TrySendError};
		let mut next_packet: Option<laminar::Packet> = None;
		loop {
			profiling::scope!("send_packets");
			if exit_flag.load(atomic::Ordering::Relaxed) {
				break;
			}
			if next_packet.is_none() {
				match socknet_to_laminar_receiver.try_recv() {
					// found event, add to queue and continue the loop
					Ok(socknet_packet) => {
						profiling::scope!("send_packet", &format!("{:?}", socknet_packet));

						next_packet = Some(socknet_packet.into());
					}
					// no events, continue the loop after a short nap
					Err(TryRecvError::Empty) => {}
					// If disconnected, then kill the thread
					Err(TryRecvError::Disconnected) => break,
				}
			}
			if let Some(packet) = next_packet.take() {
				match sender.try_send(packet) {
					Ok(_) => {} // success case is no-op
					Err(TrySendError::Full(packet)) => {
						// put the packet back and wait for next loop to try to send again
						next_packet = Some(packet);
					}
					Err(TrySendError::Disconnected(_packet)) => break,
				}
			}
			thread::sleep(Duration::from_millis(1));
		}
		log::debug!(target: crate::LOG, "Dispatch thread has concluded");
	}

	pub fn channel(&self) -> &crossbeam_channel::Sender<packet::Packet> {
		&self.sender
	}

	pub fn kick(&self, address: std::net::SocketAddr) -> anyhow::Result<()> {
		self.internal_sender
			.try_send(crate::InternalMessage::DropConnection(address))?;
		Ok(())
	}
}

impl Drop for Queue {
	fn drop(&mut self) {
		self.thread_send_packets.take().unwrap().join().unwrap();
	}
}
