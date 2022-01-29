#[doc(hidden)]
mod send;
pub use send::*;

#[doc(hidden)]
mod recv;
pub use recv::*;

pub type Bidirectional = (Send, Recv);

/// An amalgamation of streams used by receivers.
/// This is used internally to help ferry stream objects from connection reception through the handler registry.
pub enum Kind {
	Unidirectional(Recv),
	Bidirectional(Send, Recv),
	Datagram(RecvBytes),
}

impl From<quinn::RecvStream> for Kind {
	fn from(recv_stream: quinn::RecvStream) -> Self {
		Self::Unidirectional(recv_stream.into())
	}
}

impl From<(quinn::SendStream, quinn::RecvStream)> for Kind {
	fn from((send, recv): (quinn::SendStream, quinn::RecvStream)) -> Self {
		Self::Bidirectional(send.into(), recv.into())
	}
}

impl From<bytes::Bytes> for Kind {
	fn from(bytes: bytes::Bytes) -> Self {
		Self::Datagram(bytes.into())
	}
}

impl Kind {
	pub async fn read_handler_id(&mut self) -> anyhow::Result<String> {
		Ok(match self {
			Self::Unidirectional(recv) => recv.read::<String>().await?,
			Self::Bidirectional(_send, recv) => recv.read::<String>().await?,
			Self::Datagram(bytes) => bytes.read::<String>().await?,
		})
	}
}
