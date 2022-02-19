use crate::{
	stream::kind::recv::{Read, Recv},
	utility::PinFutureResultLifetime,
};

pub struct Remote(quinn::RecvStream);

impl From<quinn::RecvStream> for Remote {
	fn from(stream: quinn::RecvStream) -> Self {
		Self(stream)
	}
}

impl Read for Remote {
	/// Reads an explicit number of bytes from the stream.
	///
	/// Mirrors [`write_exact`](crate::stream::kind::Write::write_exact).
	///
	/// See [`quinn`](quinn::RecvStream::read_exact) for more details.
	fn read_exact<'a>(&'a mut self, byte_count: usize) -> PinFutureResultLifetime<'a, Vec<u8>> {
		Box::pin(async move {
			let mut bytes = vec![0; byte_count];
			self.0.read_exact(&mut bytes).await?;
			Ok(bytes)
		})
	}
}

impl Recv for Remote {
	/// Stop accepting data. Discards unread data and notifies the peer to stop transmitting.
	///
	/// See [`quinn`](quinn::RecvStream::stop) for more.
	fn stop<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move { Ok(self.0.stop(quinn::VarInt::from_u32(0))?) })
	}
}
