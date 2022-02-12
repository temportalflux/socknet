use crate::{connection::Connection, stream};
use anyhow::Context;
use std::{collections::HashMap, sync::Arc};

type AnyArc = Arc<dyn std::any::Any + Send + Sync + 'static>;
struct Registered {
	identifier: AnyArc,
	fn_process: Box<
		dyn Fn(Arc<Connection>, stream::kind::Kind) -> anyhow::Result<()> + Send + Sync + 'static,
	>,
}
impl<T> From<T> for Registered
where
	T: stream::Identifier + Send + Sync + 'static,
	<T as stream::Identifier>::RecvBuilder: stream::recv::AppContext + Send + Sync + 'static,
	<<T as stream::Identifier>::RecvBuilder as stream::recv::AppContext>::Receiver:
		stream::handler::Receiver
			+ From<stream::recv::Context<<T as stream::Identifier>::RecvBuilder>>,
{
	fn from(other: T) -> Self {
		let recv_builder = other.recv_builder().clone();
		Self {
			identifier: Arc::new(other),
			fn_process: Box::new(move |connection, stream| {
				use stream::recv::AppContext;
				let builder = recv_builder.clone();
				let context = builder.into_context(connection, stream)?;
				<T as stream::Identifier>::RecvBuilder::process(context);
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
		(self.fn_process)(connection, stream)
	}
}

/// Repository of [`builders`](stream::recv::Builder) for [`receivers`](stream::handler::Receiver).
///
/// Used to read the [`unique_id`](stream::Identifier::unique_id) from an incoming stream
/// and hand off the stream handling to a unique receiver of a given type.
pub struct Registry {
	/// The map of [`unique ids`](stream::Identifier::unique_id) to the registration for all registered builders.
	registrations: HashMap<&'static str, Registered>,
}

impl Default for Registry {
	fn default() -> Self {
		Self {
			registrations: HashMap::new(),
		}
	}
}

impl Registry {
	/// Registers some [`builder`](stream::recv::Builder) so that it can create a
	/// [`receiver`](stream::handler::Receiver) when a packet with the provided id is received.
	pub fn register<T>(&mut self, identifier: T)
	where
		T: stream::Identifier + Send + Sync + 'static,
		<T as stream::Identifier>::RecvBuilder: stream::recv::AppContext + Send + Sync + 'static,
		<<T as stream::Identifier>::RecvBuilder as stream::recv::AppContext>::Receiver:
			stream::handler::Receiver
				+ From<stream::recv::Context<<T as stream::Identifier>::RecvBuilder>>,
	{
		self.registrations
			.insert(T::unique_id(), Registered::from(identifier));
	}

	/// Finds a builder based on the id of a given identifier.
	pub fn get<T>(self: &Arc<Registry>) -> anyhow::Result<Arc<T>>
	where
		T: stream::Identifier + Send + Sync + 'static,
	{
		let id: &'static str = <T as stream::Identifier>::unique_id();
		let reg = self
			.registrations
			.get(&id)
			.ok_or(Error::NoSuchRegistration(id))?;
		let identifier = reg
			.identifier
			.clone()
			.downcast::<T>()
			.map_err(|_| Error::RegistrationTypeMismatch(id))?;
		Ok(identifier)
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
		let log = connection.log_target();
		crate::utility::spawn(log.clone(), async move {
			let handler_id = stream
				.read_handler_id()
				.await
				.context("reading handler id")?;
			match self.registrations.get(handler_id.as_str()) {
				Some(registered) => {
					registered.process(connection, stream)?;
				}
				None => {
					log::error!(
						target: &log,
						"Failed to find stream handler for id {}",
						handler_id
					);
				}
			}
			Ok(())
		});
	}
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("There is no registered identifier for id({0}).")]
	NoSuchRegistration(&'static str),
	#[error("Tried to get registered identifier for id({0}), but the registration could not be downcast to the provided type.")]
	RegistrationTypeMismatch(&'static str),
}
