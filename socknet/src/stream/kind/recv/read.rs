use crate::utility::PinFutureResultLifetime;

/// Local interface for providing the same async-read functionality to both
/// [`ongoing`](crate::stream::kind::Recv) and [`datagram`](crate::stream::kind::RecvBytes) streams.
pub trait Read {
	/// Reads an explicit number of bytes from the stream.
	///
	/// Mirrors [`write_exact`](crate::stream::kind::Write::write_exact).
	fn read_exact<'a>(&'a mut self, byte_count: usize) -> PinFutureResultLifetime<'a, Vec<u8>>;

	/// Reads a usize from the stream, encoded at a specific byte length.
	///
	/// Used primarily internally to read the sizes of byte vecs and generic structs to the stream.
	///
	/// Mirrors [`write_size`](crate::stream::kind::Write::write_size).
	fn read_size<'a>(&'a mut self) -> PinFutureResultLifetime<'a, usize>
	where
		Self: Send,
	{
		Box::pin(async move {
			let len_encoded_size = std::mem::size_of::<u32>();
			let encoded = self.read_exact(len_encoded_size).await?;
			let size: u32 = bincode::deserialize(&encoded[..])?;
			Ok(size as usize)
		})
	}

	/// Reads a set of bytes as a distinct vec, prefixed with a size header.
	///
	/// This is different than [`read_exact`](Self::read_exact) because it reads a distinct header between set of bytes.
	///
	/// Mirrors [`write_bytes`](crate::stream::kind::Write::write_bytes).
	fn read_bytes<'a>(&'a mut self) -> PinFutureResultLifetime<'a, Vec<u8>>
	where
		Self: Send,
	{
		Box::pin(async move {
			let byte_count = self.read_size().await?;
			self.read_exact(byte_count).await
		})
	}

	/// Reads some generic sized data from the stream, prefixed with a size header.
	///
	/// Mirrors [`write`](crate::stream::kind::Write::write).
	fn read<'a, T>(&'a mut self) -> PinFutureResultLifetime<'a, T>
	where
		Self: Send,
		T: serde::de::DeserializeOwned + Sized + Send + Sync + 'static,
	{
		Box::pin(async move {
			let encoded = self.read_bytes().await?;
			// Convert data bytes to type
			let data: T = bincode::deserialize(&encoded[..])?;
			Ok(data)
		})
	}
}
