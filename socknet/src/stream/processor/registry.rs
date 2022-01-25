use crate::{
	connection::Connection,
	stream::{
		processor::{Processor, Registerable, Registration},
		Typed,
	},
};
use std::{collections::HashMap, sync::Arc};

pub struct Registry {
	handlers: HashMap<&'static str, Registration>,
}

impl Default for Registry {
	fn default() -> Self {
		Self {
			handlers: HashMap::new(),
		}
	}
}

impl Registry {
	pub fn register<T>(&mut self)
	where
		T: Registerable + Send + Sync + 'static,
	{
		self.handlers
			.insert(T::unique_id(), Registration::new::<T>());
	}
}

impl Processor for Registry {
	fn create_receiver(self: Arc<Self>, connection: Arc<Connection>, mut stream: Typed) {
		connection.clone().spawn(async move {
			let handler_id = stream.read_handler_id().await?;
			match self.handlers.get(handler_id.as_str()) {
				Some(registration) => {
					registration.create_receiver(Arc::downgrade(&connection), stream)?;
				}
				None => {
					log::error!(
						target: crate::LOG,
						"Failed to find stream handler for id {}",
						handler_id
					);
				}
			}
			Ok(())
		});
	}
}
