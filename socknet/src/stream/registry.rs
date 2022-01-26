use crate::{
	connection::Connection,
	stream::{Buildable, Builder, Result, Typed},
};
use anyhow::Context;
use std::{collections::HashMap, sync::Arc};

type AnyBuilder = Arc<dyn std::any::Any + Send + Sync + 'static>;
struct Registered {
	builder: AnyBuilder,
	fn_build: Box<dyn Fn(AnyBuilder, Arc<Connection>, Typed) -> Result<()> + Send + Sync + 'static>,
}
impl<T> From<T> for Registered
where
	T: Builder + Send + Sync + 'static,
{
	fn from(other: T) -> Self {
		Self {
			builder: Arc::new(other),
			fn_build: Box::new(|arc, connection, stream| {
				arc.downcast::<T>().unwrap().build(connection, stream)
			}),
		}
	}
}
impl Registered {
	fn build(&self, connection: Arc<Connection>, stream: Typed) -> Result<()> {
		(self.fn_build)(self.builder.clone(), connection, stream)
	}

	pub fn get<T>(&self) -> Option<Arc<T>>
	where
		T: Builder + Send + Sync + 'static,
	{
		self.builder.clone().downcast::<T>().ok()
	}
}

pub struct Registry {
	builders: HashMap<&'static str, Registered>,
}

impl Default for Registry {
	fn default() -> Self {
		Self {
			builders: HashMap::new(),
		}
	}
}

impl Registry {
	pub fn register<T>(&mut self, builder: T)
	where
		T: Builder + Send + Sync + 'static,
	{
		self.builders
			.insert(T::unique_id(), Registered::from(builder));
	}

	pub fn get<T>(self: &Arc<Registry>) -> Option<Arc<T>>
	where
		T: Builder + Send + Sync + 'static,
	{
		self.builders
			.get(&T::unique_id())
			.map(|reg| reg.get::<T>())
			.flatten()
	}

	pub fn builder<T>(self: &Arc<Self>) -> Option<Arc<T::Builder>>
	where
		T: Buildable,
		T::Builder: Builder + Send + Sync + 'static,
	{
		self.get::<T::Builder>()
	}
}

impl Registry {
	pub(crate) fn create_receiver(self: Arc<Self>, connection: Arc<Connection>, mut stream: Typed) {
		crate::utility::spawn(connection.log_target(), async move {
			let handler_id = stream
				.read_handler_id()
				.await
				.context("reading handler id")?;
			match self.builders.get(handler_id.as_str()) {
				Some(registered) => {
					registered.build(connection, stream)?;
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
