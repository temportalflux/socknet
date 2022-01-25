use crate::{
	connection::Connection,
	stream::{self, processor::Registerable},
};
use std::sync::Weak;

type FnCreateReceiver =
	Box<dyn Fn(Weak<Connection>, stream::Typed) -> anyhow::Result<()> + Send + Sync + 'static>;
pub(crate) struct Registration {
	fn_create_receiver: FnCreateReceiver,
}

impl Registration {
	pub fn new<T>() -> Self
	where
		T: Registerable + Send + Sync + 'static,
	{
		Self {
			fn_create_receiver: Box::new(|connection, stream| {
				T::create_receiver(connection, stream)
			}),
		}
	}

	pub fn create_receiver(
		&self,
		connection: Weak<Connection>,
		stream: stream::Typed,
	) -> anyhow::Result<()> {
		(self.fn_create_receiver)(connection, stream)
	}
}
