use crate::{
	stream::{
		kind::recv::{Read, Recv},
		local,
	},
	utility::PinFutureResultLifetime,
};

pub(crate) type Internal = async_channel::Receiver<local::AnyBox>;
pub struct Local(Internal);

impl From<Internal> for Local {
	fn from(stream: Internal) -> Self {
		Self(stream)
	}
}

impl Local {
	fn read_any<'a, T>(&'a mut self) -> PinFutureResultLifetime<'a, T>
	where
		T: 'static + Send + Sync,
	{
		Box::pin(async move {
			let any = self.0.recv().await?;
			let byte_vec = any
				.downcast::<T>()
				.map_err(|_| LocalError::InvalidTypeEncountered)?;
			Ok(*byte_vec)
		})
	}
}

impl Read for Local {
	fn read_exact<'a>(&'a mut self, byte_count: usize) -> PinFutureResultLifetime<'a, Vec<u8>> {
		Box::pin(async move {
			let byte_vec = self.read_any::<Vec<u8>>().await?;
			assert_eq!(byte_vec.len(), byte_count);
			Ok(byte_vec)
		})
	}

	fn read_size<'a>(&'a mut self) -> PinFutureResultLifetime<'a, usize> {
		self.read_any::<usize>()
	}

	fn read_bytes<'a>(&'a mut self) -> PinFutureResultLifetime<'a, Vec<u8>> {
		self.read_any::<Vec<u8>>()
	}

	fn read<'a, T>(&'a mut self) -> PinFutureResultLifetime<'a, T>
	where
		T: serde::de::DeserializeOwned + Sized + Send + Sync + 'static,
	{
		self.read_any::<T>()
	}
}

impl Recv for Local {
	fn stop<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.close();
			Ok(())
		})
	}
}

#[derive(thiserror::Error, Debug)]
pub enum LocalError {
	#[error("Encountered a type in the stream which did not match the expected type")]
	InvalidTypeEncountered,
}
