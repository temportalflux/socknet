use crate::stream;
use std::net::SocketAddr;

pub type Sender = async_channel::Sender<Error>;
pub type Receiver = async_channel::Receiver<Error>;
pub struct Error {
	pub address: SocketAddr,
	pub kind: stream::Kind,
	pub error: quinn::ConnectionError,
}

impl std::error::Error for Error {}
impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(
			f,
			"Encountered error from {} for {} stream: {}",
			self.address, self.kind, self.error
		)
	}
}
