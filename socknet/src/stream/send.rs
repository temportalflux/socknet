use std::sync::Arc;

use crate::{connection::Connection, stream, utility::PinFutureResult};

/// Contextual information about an outgoing stream.
/// Provided to newly created [`Initiator`](stream::handler::Initiator) structs to
/// carry application context, connection, and stream objects.
#[allow(type_alias_bounds)]
pub type Context<T: AppContext> = stream::Context<T, <T::Opener as stream::Opener>::Output>;

/// A builder for creating handlers which implement the [`Initiator`](stream::handler::Initiator) trait.
/// Implementors must create at least 1 builder per initiator type
/// (which may be shared with the [`Receiver`](stream::handler::Receiver) type).
pub trait AppContext {
	/// The kind of stream that this builder should create.
	type Opener: stream::Opener;

	fn open(
		connection: Arc<Connection>,
	) -> PinFutureResult<<Self::Opener as stream::Opener>::Output> {
		<Self::Opener as stream::Opener>::open(connection)
	}
}
