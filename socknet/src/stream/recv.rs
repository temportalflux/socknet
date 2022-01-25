pub struct Recv(pub(crate) quinn::RecvStream);

impl From<quinn::RecvStream> for Recv {
	fn from(stream: quinn::RecvStream) -> Self {
		Self(stream)
	}
}

impl Recv {
	pub async fn read_exact(&mut self, byte_count: usize) -> anyhow::Result<Vec<u8>> {
		let mut bytes = vec![0; byte_count];
		self.0.read_exact(&mut bytes).await?;
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

	pub async fn stop(&mut self, code: u32) -> anyhow::Result<()> {
		Ok(self.0.stop(quinn::VarInt::from_u32(code))?)
	}
}
