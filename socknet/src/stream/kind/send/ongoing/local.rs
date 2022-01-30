use crate::{
	stream::{
		kind::send::{Send, Write},
		local,
	},
	utility::PinFutureResultLifetime,
};

pub(crate) type Internal = async_channel::Sender<local::AnyBox>;
pub struct Local(Internal);

impl From<Internal> for Local {
	fn from(stream: Internal) -> Self {
		Self(stream)
	}
}

impl Local {
	fn write_any<'a, T>(&'a mut self, any: T) -> PinFutureResultLifetime<'a, ()>
	where
		T: std::marker::Send + Sync + 'static,
	{
		Box::pin(async move {
			self.0.send(Box::new(any)).await?;
			Ok(())
		})
	}
}

impl Write for Local {
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		self.write_any(buf.to_vec())
	}

	fn write_size<'a>(&'a mut self, len: usize) -> PinFutureResultLifetime<'a, ()> {
		self.write_any(len)
	}

	fn write_bytes<'a>(&'a mut self, data: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		self.write_any(data.to_vec())
	}

	fn write<'a, T>(&'a mut self, data: &'a T) -> PinFutureResultLifetime<'a, ()>
	where
		Self: std::marker::Send,
		T: 'static + serde::Serialize + Clone + std::marker::Send + Sync,
	{
		self.write_any(data.clone())
	}
}

impl Send for Local {
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move { Ok(()) })
	}
}
