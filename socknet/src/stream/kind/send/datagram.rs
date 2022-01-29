use crate::{connection::Connection, stream::kind::Write, utility::PinFutureResultLifetime};
use std::sync::Arc;

/// An outgoing buffer of bytes which is sent to its connection when [`finish`](Self::finish) is called.
///
/// IMPORTANT: This stream does not operate like a [`Send stream`](crate::stream::kind::Send).
/// You MUST call [`finish`](Self::finish) to transmit the datagram (unreliably) to the peer.
/// See [`quinn::Connection::send_datagram`](quinn::Connection::send_datagram) for more information.
pub struct SendBytes(pub(crate) Vec<u8>, pub(crate) Arc<Connection>);

impl Write for SendBytes {
	/// Writes all of the provided bytes to the stream.
	///
	/// This operation has no internal awaits, and appends the byte buf to the internal buffer.
	///
	/// Mirrors [`read_exact`](crate::stream::kind::Read::read_exact).
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.extend_from_slice(&buf);
			Ok(())
		})
	}

	/// Finishes the stream by calling [`send_datagram`](Connection::send_datagram) on the held connection.
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			use bytes::Bytes;
			let bytes = self.0.drain(..).collect::<Vec<_>>();
			let bytes = Bytes::from(bytes);
			self.1.send_datagram(bytes)?;
			Ok(())
		})
	}
}
