use crate::{
	stream::kind::send::{Send, Write},
	utility::PinFutureResultLifetime,
};

pub struct Remote(quinn::SendStream);

impl From<quinn::SendStream> for Remote {
	fn from(stream: quinn::SendStream) -> Self {
		Self(stream)
	}
}

impl Write for Remote {
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
}

impl Send for Remote {
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
