use crate::{
	connection::Connection,
	stream::kind::send::{Send, Write},
	utility::PinFutureResultLifetime,
};
use std::sync::Arc;

pub struct Remote(pub(crate) Vec<u8>, pub(crate) Arc<Connection>);

impl Write for Remote {
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
}

impl Send for Remote {
	/// Finishes the stream by calling [`send_datagram`](Connection::send_datagram) on the held connection.
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			use bytes::Bytes;
			let bytes = self.0.drain(..).collect::<Vec<_>>();
			let bytes = Bytes::from(bytes);
			self.1.send_datagram(bytes.into())?;
			Ok(())
		})
	}
}
