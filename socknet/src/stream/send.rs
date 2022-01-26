use crate::stream::Builder;

pub struct Send(pub(crate) quinn::SendStream);

impl From<quinn::SendStream> for Send {
	fn from(stream: quinn::SendStream) -> Self {
		Self(stream)
	}
}

impl Send {
	pub async fn write_id<T>(&mut self) -> anyhow::Result<()>
	where
		T: Builder + std::marker::Send + Sync + 'static,
	{
		self.write(&T::unique_id().to_owned()).await
	}

	async fn write_size(&mut self, len: usize) -> anyhow::Result<()> {
		let len = len as u32;
		let len_encoded = bincode::serialize(&len)?;
		self.0.write_all(&len_encoded).await?;
		Ok(())
	}

	pub async fn write<T>(&mut self, data: &T) -> anyhow::Result<()>
	where
		T: serde::Serialize,
	{
		// The data to be sent, in serialized to bytes
		let data_encoded = bincode::serialize(&data)?;
		self.write_size(data_encoded.len()).await?;
		// Write the data to the buffer
		self.0.write_all(&data_encoded).await?;
		Ok(())
	}

	pub async fn write_bytes(&mut self, data: &[u8]) -> anyhow::Result<()> {
		self.write_size(data.len()).await?;
		self.0.write_all(data).await?;
		Ok(())
	}

	pub async fn finish(&mut self) -> anyhow::Result<()> {
		Ok(self.0.finish().await?)
	}

	pub async fn stopped(&mut self) -> anyhow::Result<u32> {
		let code = self.0.stopped().await?;
		Ok(code.into_inner() as u32)
	}
}
