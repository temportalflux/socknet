use crate::{
	connection::{active::Active, Connection},
	stream::{
		kind::send::{Send, Write},
		local,
	},
	utility::PinFutureResultLifetime,
};
use std::sync::Arc;

pub struct Local(pub(crate) Vec<local::AnyBox>, pub(crate) Arc<Connection>);

impl Write for Local {
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.push(Box::new(buf.to_vec()));
			Ok(())
		})
	}

	fn write_size<'a>(&'a mut self, len: usize) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.push(Box::new(len));
			Ok(())
		})
	}

	fn write_bytes<'a>(&'a mut self, data: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			self.0.push(Box::new(data.to_vec()));
			Ok(())
		})
	}

	fn write<'a, T>(&'a mut self, data: &'a T) -> PinFutureResultLifetime<'a, ()>
	where
		Self: std::marker::Send,
		T: 'static + serde::Serialize + Clone + std::marker::Send + Sync,
	{
		Box::pin(async move {
			self.0.push(Box::new(data.clone()));
			Ok(())
		})
	}
}

impl Send for Local {
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		Box::pin(async move {
			let items = self.0.drain(..).collect::<Vec<_>>();
			self.1.send_datagram(items.into())?;
			Ok(())
		})
	}
}
