use crate::stream;

pub enum Typed {
	Unidirectional(quinn::RecvStream),
	Bidirectional(quinn::SendStream, quinn::RecvStream),
	Datagram(stream::Bytes),
}

impl From<quinn::RecvStream> for Typed {
	fn from(recv_stream: quinn::RecvStream) -> Self {
		Self::Unidirectional(recv_stream)
	}
}

impl From<(quinn::SendStream, quinn::RecvStream)> for Typed {
	fn from((send, recv): (quinn::SendStream, quinn::RecvStream)) -> Self {
		Self::Bidirectional(send, recv)
	}
}

impl From<stream::Bytes> for Typed {
	fn from(bytes: stream::Bytes) -> Self {
		Self::Datagram(bytes)
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

	pub async fn read<'a, T>(&mut self) -> anyhow::Result<T>
	where
		T: serde::de::DeserializeOwned + Sized,
	{
		let size_of_t = std::mem::size_of::<T>();
		let bytes = match self {
			stream::Typed::Unidirectional(recv) => {
				let mut bytes = Vec::with_capacity(size_of_t);
				recv.read_exact(&mut bytes).await?;
				bytes
			}
			stream::Typed::Bidirectional(_send, recv) => {
				let mut bytes = Vec::with_capacity(size_of_t);
				recv.read_exact(&mut bytes).await?;
				bytes
			}
			stream::Typed::Datagram(bytes) => {
				let t_bytes = bytes.split_to(size_of_t);
				t_bytes.to_vec()
			}
		};
		Ok(rmp_serde::from_read(std::io::Cursor::new(&bytes))?)
	}
}
