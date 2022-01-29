use crate::{stream::kind::Write, utility::PinFutureResultLifetime};

/// An outgoing stream that can continue to send data as long as the connection is available.
/// Once opened, [`write`](Write) methods can be used to transmit data via a [`SendStream`](quinn::SendStream).
pub struct Send(pub(crate) quinn::SendStream);

impl From<quinn::SendStream> for Send {
	fn from(stream: quinn::SendStream) -> Self {
		Self(stream)
	}
}

impl Write for Send {
	/// Writes all of the provided bytes to the stream.
	///
	/// Mirrors [`read_exact`](crate::stream::kind::Read::read_exact).
	///
	/// See [`quinn`](quinn::SendStream::write_all) for more details.
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.write_all(&buf).await?;
			Ok(())
		})
	}

	/// Finishes the stream.
	///
	/// See [`quinn`](quinn::SendStream::finish) for more details.
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.finish().await?;
			Ok(())
		})
	}
}

impl Send {
	/// Completes if/when the peer stops the stream, yielding the error code.
	///
	/// Mirrors [`stop`](crate::stream::kind::Recv::stop).
	///
	/// See [`quinn`](quinn::SendStream::stopped) for more.
	pub async fn stopped(&mut self) -> anyhow::Result<u32> {
		let code = self.0.stopped().await?;
		Ok(code.into_inner() as u32)
	}
}
