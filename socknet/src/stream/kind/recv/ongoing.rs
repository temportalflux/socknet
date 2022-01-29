use crate::{stream::kind::Read, utility::PinFutureResultLifetime};

/// An incoming stream that can continue to read data as long as the connection is available.
/// Once opened, [`read`](Read) methods can be used to receive data via a [`RecvStream`](quinn::RecvStream).
pub struct Recv(quinn::RecvStream);

impl From<quinn::RecvStream> for Recv {
	fn from(stream: quinn::RecvStream) -> Self {
		Self(stream)
	}
}

impl Read for Recv {
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

impl Recv {
	/// Stop accepting data. Discards unread data and notifies the peer to stop transmitting.
	///
	/// Mirrors [`stopped`](crate::stream::kind::Send::stopped).
	///
	/// See [`quinn`](quinn::RecvStream::stop) for more.
	pub async fn stop(&mut self, code: u32) -> anyhow::Result<()> {
		Ok(self.0.stop(quinn::VarInt::from_u32(code))?)
	}
}
