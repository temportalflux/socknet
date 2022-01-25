use crate::stream::processor::Registerable;

pub struct Send(pub(crate) quinn::SendStream);

impl From<quinn::SendStream> for Send {
	fn from(stream: quinn::SendStream) -> Self {
		Self(stream)
	}
}

impl Send {
	pub async fn write_id<T>(&mut self) -> anyhow::Result<()>
	where
		T: Registerable + std::marker::Send + Sync + 'static,
	{
		self.write(&T::unique_id().to_owned()).await
	}

	pub async fn write<T>(&mut self, data: &T) -> anyhow::Result<()>
	where
		T: serde::Serialize,
	{
		// The data to be sent, in serialized to bytes
		let data_encoded = bincode::serialize(&data)?;
		// The number of bytes it took to serialize the data
		let byte_count = data_encoded.len() as u32;
		// The number of bytes to send, serialized to bytes
		let len_encoded = bincode::serialize(&byte_count)?;
		// Write the number of bytes to the buffer
		self.0.write_all(&len_encoded).await?;
		// Write the data to the buffer
		self.0.write_all(&data_encoded).await?;
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
