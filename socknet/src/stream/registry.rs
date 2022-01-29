use crate::{connection::Connection, stream};
use anyhow::Context;
use std::{collections::HashMap, sync::Arc};

type AnyBuilder = Arc<dyn std::any::Any + Send + Sync + 'static>;
struct Registered {
	builder: AnyBuilder,
	fn_process: Box<
		dyn Fn(AnyBuilder, Arc<Connection>, stream::kind::Kind) -> anyhow::Result<()>
			+ Send
			+ Sync
			+ 'static,
	>,
}
impl<T> From<T> for Registered
where
	T: stream::recv::Builder + Send + Sync + 'static,
	T::Receiver: From<stream::recv::Context<T>> + stream::handler::Receiver,
{
	fn from(other: T) -> Self {
		Self {
			builder: Arc::new(other),
			fn_process: Box::new(|any, connection, stream| {
				let builder = any.downcast::<T>().unwrap();
				let context = builder.into_context(connection, stream)?;
				T::process(context);
				Ok(())
			}),
		}
	}
}
impl Registered {
	fn process(
		&self,
		connection: Arc<Connection>,
		stream: stream::kind::Kind,
	) -> anyhow::Result<()> {
		(self.fn_process)(self.builder.clone(), connection, stream)
	}

	pub fn get<T>(&self) -> Option<Arc<T>>
	where
		T: Send + Sync + 'static,
	{
		self.builder.clone().downcast::<T>().ok()
	}
}

/// Repository of [`builders`](stream::recv::Builder) for [`receivers`](stream::handler::Receiver).
///
/// Used to read the [`unique_id`](stream::Identifier::unique_id) from an incoming stream
/// and hand off the stream handling to a unique receiver of a given type.
pub struct Registry {
	/// The map of [`unique ids`](stream::Identifier::unique_id) to the registration for all registered builders.
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
	/// Registers some [`builder`](stream::recv::Builder) so that it can create a
	/// [`receiver`](stream::handler::Receiver) when a packet with the provided id is received.
	pub fn register<T>(&mut self, builder: T)
	where
		T: stream::recv::Builder + Send + Sync + 'static,
		T::Receiver: From<stream::recv::Context<T>> + stream::handler::Receiver,
	{
		self.builders
			.insert(T::unique_id(), Registered::from(builder));
	}

	/// Finds a builder based on the id of a given identifier.
	pub fn get<T>(self: &Arc<Registry>) -> Option<Arc<T>>
	where
		T: stream::Identifier + Send + Sync + 'static,
	{
		self.builders
			.get(&<T as stream::Identifier>::unique_id())
			.map(|reg| reg.get::<T>())
			.flatten()
	}

	/// Returns the builder instance, if it was registered.
	pub fn builder<T>(self: &Arc<Self>) -> Option<Arc<T::Builder>>
	where
		T: stream::handler::Receiver,
		T::Builder: Send + Sync + 'static,
	{
		self.get::<T::Builder>()
	}
}

impl Registry {
	/// Creates the receiver and spawns the process for an incoming stream of any kind.
	///
	/// This function spawns its own async task/future, so all passed params
	/// will start to be processed but the call itself is non-blocking.
	pub(crate) fn create_receiver(
		self: Arc<Self>,
		connection: Arc<Connection>,
		mut stream: stream::kind::Kind,
	) {
		crate::utility::spawn(connection.log_target(), async move {
			let handler_id = stream
				.read_handler_id()
				.await
				.context("reading handler id")?;
			match self.builders.get(handler_id.as_str()) {
				Some(registered) => {
					registered.process(connection, stream)?;
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
