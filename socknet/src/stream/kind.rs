use crate::stream::local;

mod locality;
pub use locality::*;

pub mod recv;
pub use recv::{Read, Recv};

pub mod send;
pub use send::{Send, Write};

/// An amalgamation of streams used by receivers.
/// This is used internally to help ferry stream objects from connection reception through the handler registry.
pub enum Kind {
	Unidirectional(recv::Ongoing),
	Bidirectional(Bidirectional),
	Datagram(recv::Datagram),
}
pub type Bidirectional = (send::Ongoing, recv::Ongoing);

impl From<quinn::RecvStream> for Kind {
	fn from(recv_stream: quinn::RecvStream) -> Self {
		Self::Unidirectional(recv_stream.into())
	}
}

impl From<(quinn::SendStream, quinn::RecvStream)> for Kind {
	fn from((send, recv): (quinn::SendStream, quinn::RecvStream)) -> Self {
		Self::Bidirectional((send.into(), recv.into()))
	}
}

impl From<bytes::Bytes> for Kind {
	fn from(bytes: bytes::Bytes) -> Self {
		Self::Datagram(bytes.into())
	}
}

impl From<recv::ongoing::local::Internal> for Kind {
	fn from(recv: recv::ongoing::local::Internal) -> Self {
		Self::Unidirectional(recv.into())
	}
}

impl
	From<(
		send::ongoing::local::Internal,
		recv::ongoing::local::Internal,
	)> for Kind
{
	fn from(
		(send, recv): (
			send::ongoing::local::Internal,
			recv::ongoing::local::Internal,
		),
	) -> Self {
		Self::Bidirectional((send.into(), recv.into()))
	}
}

impl From<Vec<local::AnyBox>> for Kind {
	fn from(data: Vec<local::AnyBox>) -> Self {
		Self::Datagram(data.into())
	}
}

impl Kind {
	pub async fn read_handler_id(&mut self) -> anyhow::Result<String> {
		Ok(match self {
			Self::Unidirectional(recv) => recv.read::<String>().await?,
			Self::Bidirectional((_send, recv)) => recv.read::<String>().await?,
			Self::Datagram(recv) => recv.read::<String>().await?,
		})
	}
}
