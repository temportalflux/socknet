use crate::stream;

pub enum Typed {
	Unidirectional(stream::Recv),
	Bidirectional(stream::Send, stream::Recv),
	Datagram(stream::Bytes),
}

impl From<quinn::RecvStream> for Typed {
	fn from(recv_stream: quinn::RecvStream) -> Self {
		Self::Unidirectional(recv_stream.into())
	}
}

impl From<(quinn::SendStream, quinn::RecvStream)> for Typed {
	fn from((send, recv): (quinn::SendStream, quinn::RecvStream)) -> Self {
		Self::Bidirectional(send.into(), recv.into())
	}
}

impl From<bytes::Bytes> for Typed {
	fn from(bytes: bytes::Bytes) -> Self {
		Self::Datagram(bytes.into())
	}
}

impl Typed {
	pub fn kind(&self) -> stream::Kind {
		match &self {
			Self::Unidirectional(_) => stream::Kind::Unidirectional,
			Self::Bidirectional(_, _) => stream::Kind::Bidirectional,
			Self::Datagram(_) => stream::Kind::Datagram,
		}
	}

	pub async fn read_handler_id(&mut self) -> anyhow::Result<String> {
		Ok(match self {
			stream::Typed::Unidirectional(recv) => recv.read::<String>().await?,
			stream::Typed::Bidirectional(_send, recv) => recv.read::<String>().await?,
			stream::Typed::Datagram(bytes) => bytes.read::<String>().await?,
		})
	}
}
