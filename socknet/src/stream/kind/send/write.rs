use crate::utility::PinFutureResultLifetime;

/// Local interface for providing the same async-write functionality to both
/// [`ongoing`](crate::stream::kind::Send) and [`datagram`](crate::stream::kind::SendBytes) streams.
pub trait Write {
	/// Writes all of the provided bytes to the stream.
	///
	/// Mirrors [`read_exact`](crate::stream::kind::Read::read_exact).
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()>;

	/// Finishes the stream.
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()>;

	/// Writes a usize to the stream, encoded at a specific byte length.
	///
	/// Used primarily internally to write the sizes of byte vecs and generic structs to the stream.
	///
	/// Mirrors [`read_size`](crate::stream::kind::Read::read_size).
	fn write_size<'a>(&'a mut self, len: usize) -> PinFutureResultLifetime<'a, ()>
	where
		Self: Send,
	{
		Box::pin(async move {
			let len = len as u32;
			let len_encoded = bincode::serialize(&len)?;
			self.write_exact(&len_encoded).await?;
			Ok(())
		})
	}

	/// Writes a set of bytes as a distinct vec, prefixing it with a size header.
	///
	/// This is different than [`write_exact`](Self::write_exact) because it provides a distinct header between set of bytes.
	///
	/// Mirrors [`read_bytes`](crate::stream::kind::Read::read_bytes).
	fn write_bytes<'a>(&'a mut self, data: &'a [u8]) -> PinFutureResultLifetime<'a, ()>
	where
		Self: Send,
	{
		Box::pin(async move {
			self.write_size(data.len()).await?;
			self.write_exact(data).await?;
			Ok(())
		})
	}

	/// Writes some generic sized data to the stream, prefixing it with a size header.
	///
	/// Mirrors [`read`](crate::stream::kind::Read::read).
	fn write<'a, T>(&'a mut self, data: &'a T) -> PinFutureResultLifetime<'a, ()>
	where
		Self: Send,
		T: serde::Serialize + Send + Sync,
	{
		Box::pin(async move {
			// The data to be sent, in serialized to bytes
			let data_encoded = bincode::serialize(&data)?;
			self.write_bytes(&data_encoded).await?;
			Ok(())
		})
	}
}
