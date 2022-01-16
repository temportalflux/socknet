use super::{DeliveryGuarantee, Guarantee, Kind, OrderGuarantee, Payload};
use std::{
	collections::HashSet,
	net::{SocketAddr, ToSocketAddrs},
};

#[derive(Clone)]
pub enum PacketMode {
	/// Packet is sent to the specified addresses.
	Directed,
	// Packet is sent to whatever the application deems as the "server".
	ToServer,
	// Packet is sent to all connections.
	Broadcast,
}

impl Default for PacketMode {
	fn default() -> Self {
		PacketMode::Directed
	}
}

pub trait AddressReference {
	fn local_address(&self) -> &SocketAddr;
	fn active_addresses(&self) -> Vec<SocketAddr>;
	fn server_address(&self) -> SocketAddr;
}

#[derive(Clone)]
pub struct PacketBuilder {
	mode: PacketMode,
	ignore_local_address: bool,
	include_addresses: HashSet<SocketAddr>,
	exclude_addresses: HashSet<SocketAddr>,
	guarantee: Guarantee,
	payloads: Vec<Payload>,
}

impl Default for PacketBuilder {
	fn default() -> Self {
		use DeliveryGuarantee::*;
		use OrderGuarantee::*;
		Self {
			mode: PacketMode::default(),
			ignore_local_address: false,
			include_addresses: HashSet::new(),
			exclude_addresses: HashSet::new(),
			guarantee: Unreliable + Unordered,
			payloads: Vec::with_capacity(1),
		}
	}
}

impl PacketBuilder {
	pub fn with_mode(mut self, mode: PacketMode) -> Self {
		self.mode = mode;
		self
	}

	fn item_as_address<T>(address: T) -> std::io::Result<SocketAddr>
	where
		T: ToSocketAddrs,
	{
		let mut iter = address.to_socket_addrs()?;
		Ok(iter.next().unwrap())
	}

	pub fn ignore_local_address(mut self) -> Self {
		self.ignore_local_address = true;
		self
	}

	pub fn with_address<T>(mut self, address: T) -> std::io::Result<Self>
	where
		T: ToSocketAddrs,
	{
		self.include_address(address)?;
		Ok(self)
	}

	pub fn include_address<T>(&mut self, address: T) -> std::io::Result<()>
	where
		T: ToSocketAddrs,
	{
		self.include_addresses
			.insert(Self::item_as_address(address)?);
		Ok(())
	}

	pub fn without_address<T>(mut self, address: T) -> std::io::Result<Self>
	where
		T: ToSocketAddrs,
	{
		self.exclude_address(address)?;
		Ok(self)
	}

	pub fn exclude_address<T>(&mut self, address: T) -> std::io::Result<()>
	where
		T: ToSocketAddrs,
	{
		self.exclude_addresses
			.insert(Self::item_as_address(address)?);
		Ok(())
	}

	pub fn with_guarantee(mut self, guarantee: Guarantee) -> Self {
		self.guarantee = guarantee;
		self
	}

	pub fn with_payload<T>(mut self, payload: &T) -> Self
	where
		T: Kind,
	{
		self.add_payload(payload);
		self
	}

	pub fn with_payloads<T>(mut self, payloads: &[T]) -> Self
	where
		T: Kind,
	{
		for payload in payloads.iter() {
			self.add_payload(payload);
		}
		self
	}

	pub fn add_payload<T>(&mut self, payload: &T)
	where
		T: Kind,
	{
		profiling::scope!("add_payload", T::unique_id());
		self.payloads.push(Payload::from(payload));
	}

	pub fn into_addresses<T: AddressReference>(&self, reference: &T) -> Vec<SocketAddr> {
		let local_address = reference.local_address();
		match self.mode {
			PacketMode::Directed => self
				.include_addresses
				.iter()
				.filter_map(|address| {
					if self.ignore_local_address && *address == *local_address {
						None
					} else {
						Some(address.clone())
					}
				})
				.collect(),
			PacketMode::ToServer => vec![reference.server_address()],
			PacketMode::Broadcast => reference
				.active_addresses()
				.into_iter()
				.filter_map(|active_address| {
					if self.ignore_local_address && active_address == *local_address {
						return None;
					}
					if self.exclude_addresses.contains(&active_address) {
						return None;
					}
					Some(active_address)
				})
				.collect(),
		}
	}

	pub fn into_packets<T: AddressReference>(self, reference: &T) -> Vec<Packet> {
		let addresses = self.into_addresses(reference);
		let mut packets = Vec::with_capacity(addresses.len() * self.payloads.len());
		for payload in self.payloads.into_iter() {
			for &address in addresses.iter() {
				packets.push(Packet {
					address,
					guarantee: self.guarantee.clone(),
					payload: payload.clone(),
				});
			}
		}
		packets
	}
}

// Mirror of `laminar::Packet`
#[derive(Clone)]
pub struct Packet {
	address: SocketAddr,
	payload: Payload,
	guarantee: Guarantee,
}

impl std::fmt::Debug for Packet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"from:{} guaranteed:({:?}, {:?}) {:?}",
			self.address,
			self.guarantee.delivery(),
			self.guarantee.order(),
			self.payload
		)
	}
}

impl From<laminar::Packet> for Packet {
	fn from(packet: laminar::Packet) -> Self {
		let payload = rmp_serde::from_read_ref::<[u8], Payload>(packet.payload()).unwrap();
		Self {
			address: packet.addr(),
			payload,
			guarantee: Guarantee {
				delivery: packet.delivery_guarantee().into(),
				order: packet.order_guarantee().into(),
			},
		}
	}
}

impl Packet {
	pub fn builder() -> PacketBuilder {
		PacketBuilder::default()
	}

	pub fn address(&self) -> &SocketAddr {
		&self.address
	}

	pub fn guarantees(&self) -> &Guarantee {
		&self.guarantee
	}

	pub fn kind(&self) -> &String {
		self.payload.kind()
	}

	pub fn take_payload(&mut self) -> Payload {
		self.payload.take()
	}
}

impl Into<laminar::Packet> for Packet {
	fn into(self) -> laminar::Packet {
		use DeliveryGuarantee::*;
		use OrderGuarantee::*;
		let raw_payload = rmp_serde::to_vec(&self.payload).unwrap();
		let payload_size = raw_payload.len();
		if payload_size > crate::socket::MAX_PACKET_SIZE {
			log::error!(
				target: crate::LOG,
				"{:?} payload size {} exceeds max packet size ({} bytes). Packet will fail.",
				self,
				payload_size,
				crate::socket::MAX_PACKET_SIZE
			);
		} else if payload_size >= crate::socket::FRAGMENT_SIZE as usize {
			log::warn!(
				target: crate::LOG,
				"{:?} payload has a size of {} which exceeds {} bytes, it will be fragmented.",
				self,
				payload_size,
				crate::socket::FRAGMENT_SIZE
			);
		}
		match self.guarantee {
			Guarantee {
				delivery: Unreliable,
				order: Unordered,
			} => laminar::Packet::unreliable(self.address, raw_payload),
			Guarantee {
				delivery: Unreliable,
				order: Sequenced,
			} => laminar::Packet::unreliable_sequenced(self.address, raw_payload, None),
			Guarantee {
				delivery: Reliable,
				order: Unordered,
			} => laminar::Packet::reliable_unordered(self.address, raw_payload),
			Guarantee {
				delivery: Reliable,
				order: Sequenced,
			} => laminar::Packet::reliable_sequenced(self.address, raw_payload, None),
			Guarantee {
				delivery: Reliable,
				order: Ordered,
			} => laminar::Packet::reliable_ordered(self.address, raw_payload, None),
			_ => panic!("Invalid guarantee {:?}", self.guarantee),
		}
	}
}
