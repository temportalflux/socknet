use crate::{stream::kind::Read, utility::PinFutureResultLifetime};

/// An incoming buffer of bytes which was sent to this connection when
/// [`finish`](crate::stream::kind::Write::finish) was called.
/// Use [`read`](Read) methods can be used to receive data, which is read from a fixed array of bytes.
pub struct RecvBytes(pub(crate) bytes::Bytes);

impl From<bytes::Bytes> for RecvBytes {
	fn from(stream: bytes::Bytes) -> Self {
		Self(stream)
	}
}

impl Read for RecvBytes {
	/// Reads an explicit number of bytes from the stream.
	///
	/// This operation has no internal awaits, and
	/// utilizes [`split_to`](bytes::Bytes::split_to) to read the next
	/// set of bytes from the internal buffer.
	///
	/// Mirrors [`write_exact`](crate::stream::kind::Write::write_exact).
	fn read_exact<'a>(&'a mut self, byte_count: usize) -> PinFutureResultLifetime<'a, Vec<u8>> {
		Box::pin(async move { Ok(self.0.split_to(byte_count).to_vec()) })
	}
}
