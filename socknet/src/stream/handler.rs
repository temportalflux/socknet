use crate::{connection::Connection, stream, utility::PinFutureResult};
use std::sync::Weak;

/// Trait implemented on a per-stream handler basis.
/// Adds functionality to open a stream for a provided connection,
/// automatically grabbing the [`builder`](Initiator::Builder) for the initiator,
/// and constructing the appropriate context.
pub trait Initiator {
	type Identifier: stream::Identifier;

	/// Opens the required stream(s) to the provided connection for this handler.
	/// Returns a future that can be awaited on, but does not do any actual processing itself.
	///
	/// In order for the receiver to know what stream handler to use,
	/// you must sent the id of the builder as the first sent item.
	fn open(connection: &Weak<Connection>) -> anyhow::Result<PinFutureResult<Self>>
	where
		Self: Sized
			+ From<stream::send::Context<<Self::Identifier as stream::Identifier>::SendBuilder>>,
		Self::Identifier: stream::Identifier + Send + Sync + 'static,
		<Self::Identifier as stream::Identifier>::SendBuilder:
			stream::send::AppContext + Send + Sync + 'static,
		<<Self::Identifier as stream::Identifier>::SendBuilder as stream::send::AppContext>::Opener:
			stream::Opener,
			<<<Self::Identifier as stream::Identifier>::SendBuilder as stream::send::AppContext>::Opener as stream::Opener>::Output: stream::kind::send::Write + Send,
	{
		let connection = Connection::upgrade(&connection)?;
		Ok(Box::pin(async move {
			use stream::Identifier;
			let registry = connection.registry()?;
			let identifier = registry.get::<Self::Identifier>()?;
			let context = identifier.open_context(connection).await?;
			Ok(Self::from(context))
		}))
	}
}

/// Trait implemented on a per-stream handler basis.
/// Adds functionality to receive a stream for a provided connection,
/// automatically grabbing the relevant builder,
/// and constructing the appropriate context in order to process the stream.
pub trait Receiver {
	type Identifier: stream::Identifier;
	/// Function called when an incoming stream has been detected and whose
	/// handler id (the first item in the stream) matched that of the associated builder's [`unique_id`](stream::Identifier::unique_id).
	///
	/// Proper reception of a stream is to spawn an async task in this function
	/// and handle all stream reading (and/or writing) in that future.
	fn receive(self);
}
