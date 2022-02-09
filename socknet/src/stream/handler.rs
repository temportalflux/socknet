use crate::{connection::Connection, stream, utility::PinFutureResult};
use std::sync::{Arc, Weak};

/// Trait implemented on a per-stream handler basis.
/// Adds functionality to open a stream for a provided connection,
/// automatically grabbing the [`builder`](Initiator::Builder) for the initiator,
/// and constructing the appropriate context.
pub trait Initiator {
	/// The type of the builder for the initiator.
	/// Used by [`get_builder`](Self::get_builder) to find the registered builder when opening a stream.
	///
	/// The builder MUST be [`registered`](stream::Registry::register) in order for this to work.
	type Builder: stream::send::Builder;

	/// Returns the builder that was registered, based on the associated type.
	fn get_builder(connection: &Arc<Connection>) -> anyhow::Result<Arc<Self::Builder>>
	where
		Self::Builder: stream::Identifier + Send + Sync + 'static,
	{
		let registry = connection.registry()?;
		Ok(registry.get::<Self::Builder>().unwrap())
	}

	/// Opens the required stream(s) to the provided connection for this handler.
	/// Returns a future that can be awaited on, but does not do any actual processing itself.
	///
	/// In order for the receiver to know what stream handler to use,
	/// you must sent the id of the builder as the first sent item.
	fn open(connection: &Weak<Connection>) -> anyhow::Result<PinFutureResult<Self>>
	where
		Self: Sized + From<stream::send::Context<Self::Builder>>,
		Self::Builder: stream::send::Builder + Send + Sync + 'static,
	{
		let connection = Connection::upgrade(&connection)?;
		Ok(Box::pin(async move {
			use stream::Identifier;
			let stream =
				<<Self::Builder as stream::send::Builder>::Opener as stream::Opener>::open(
					connection.clone(),
				)
				.await?;
			let builder = Self::get_builder(&connection)?;
			let context = stream::send::Context {
				builder,
				connection,
				stream,
			};
			Ok(Self::from(context))
		}))
	}
}

/// Trait implemented on a per-stream handler basis.
/// Adds functionality to receive a stream for a provided connection,
/// automatically grabbing the relevant [`builder`](Initiator::Builder),
/// and constructing the appropriate context in order to process the stream.
pub trait Receiver {
	/// The type of the builder for the receiver.
	/// The builder MUST be [`registered`](stream::Registry::register) in order for this to work.
	type Builder: stream::recv::Builder;

	/// Function called when an incoming stream has been detected and whose
	/// handler id (the first item in the stream) matched that of the associated builder's [`unique_id`](stream::Identifier::unique_id).
	///
	/// Proper reception of a stream is to spawn an async task in this function
	/// and handle all stream reading (and/or writing) in that future.
	fn receive(self);
}
