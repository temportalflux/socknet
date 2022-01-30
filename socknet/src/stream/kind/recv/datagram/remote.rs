use crate::{
	stream::kind::recv::{Read, Recv},
	utility::PinFutureResultLifetime,
};

pub struct Remote(bytes::Bytes);

impl From<bytes::Bytes> for Remote {
	fn from(stream: bytes::Bytes) -> Self {
		Self(stream)
	}
}

impl Read for Remote {
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

impl Recv for Remote {
	fn stop<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move { Ok(()) })
	}
}
