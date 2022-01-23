use crate::{
	stream::{
		processor::{handler::Handler, Processor},
		Stream,
	},
	utility::JoinHandleList,
};
use std::{collections::HashMap, sync::Arc};

pub struct Registry<T> {
	handlers: HashMap<T, Box<dyn Handler + Send + Sync + 'static>>,
}

impl<T> Default for Registry<T> {
	fn default() -> Self {
		Self {
			handlers: HashMap::new(),
		}
	}
}

impl Registry<usize> {
	pub fn message_handler() -> Self {
		let mut registry = Self::default();
		registry.register(0, crate::message::StreamHandler::default());
		registry
	}
}

impl<T> Registry<T>
where
	T: Eq + std::hash::Hash,
{
	pub fn register<THandler>(&mut self, id: T, handler: THandler)
	where
		THandler: Handler + Send + Sync + 'static,
	{
		self.handlers.insert(id, Box::new(handler));
	}
}

impl<T> Processor for Registry<T>
where
	T: std::fmt::Display
		+ serde::de::DeserializeOwned
		+ Eq
		+ std::hash::Hash
		+ Send
		+ Sync
		+ 'static,
{
	fn process(self: Arc<Self>, mut stream: Stream, handle_owner: Arc<JoinHandleList>) {
		handle_owner.clone().spawn(async move {
			let handler_id = stream.read::<T>().await?;
			match self.handlers.get(&handler_id) {
				Some(handler) => {
					handler.process(stream, handle_owner);
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
