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

	async fn read_size(&mut self) -> anyhow::Result<usize> {
		let len_encoded_size = std::mem::size_of::<u32>();
		let encoded = self.read_exact(len_encoded_size).await?;
		let size: u32 = bincode::deserialize(&encoded[..])?;
		Ok(size as usize)
	}

	pub async fn read<T>(&mut self) -> anyhow::Result<T>
	where
		T: serde::de::DeserializeOwned + Sized,
	{
		let byte_count = self.read_size().await?;
		// Read the data in bytes
		let encoded = self.read_exact(byte_count).await?;
		// Convert data bytes to type
		let data: T = bincode::deserialize(&encoded[..])?;
		Ok(data)
	}

	pub async fn read_bytes(&mut self) -> anyhow::Result<Vec<u8>> {
		let byte_count = self.read_size().await?;
		self.read_exact(byte_count).await
	}

	pub async fn stop(&mut self, code: u32) -> anyhow::Result<()> {
		Ok(self.0.stop(quinn::VarInt::from_u32(code))?)
	}
}
