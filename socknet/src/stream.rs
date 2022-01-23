pub use bytes::Bytes;

pub mod error;

mod kind;
pub use kind::*;

pub mod processor;

mod typed;
pub use typed::*;

pub type Sender = crossbeam_channel::Sender<Stream>;
pub type Receiver = crossbeam_channel::Receiver<Stream>;

pub struct Stream {
	pub address: std::net::SocketAddr,
	pub typed: Typed,
}

impl Stream {
	pub async fn read<'a, T>(&mut self) -> anyhow::Result<T>
	where
		T: serde::de::DeserializeOwned + Sized,
	{
		self.typed.read().await
	}
}
