pub struct Bytes(pub(crate) bytes::Bytes);

impl From<bytes::Bytes> for Bytes {
	fn from(stream: bytes::Bytes) -> Self {
		Self(stream)
	}
}

impl Bytes {
	pub async fn read_exact(&mut self, byte_count: usize) -> anyhow::Result<Vec<u8>> {
		let bytes = self.0.split_to(byte_count).to_vec();
		Ok(bytes)
	}

	pub async fn read<T>(&mut self) -> anyhow::Result<T>
	where
		T: serde::de::DeserializeOwned + Sized,
	{
		// Read the header (byytes representing the number of bytes it took to serialize the data)
		let len_encoded_size = std::mem::size_of::<u32>();
		let encoded = self.read_exact(len_encoded_size).await?;
		// The number of bytes it took to serialize the data
		let byte_count: u32 = bincode::deserialize(&encoded[..])?;
		// Read the data in bytes
		let encoded = self.read_exact(byte_count as usize).await?;
		// Convert data bytes to type
		let data: T = bincode::deserialize(&encoded[..])?;
		Ok(data)
	}
}
