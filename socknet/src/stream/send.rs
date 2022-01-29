use crate::stream;

/// Contextual information about an outgoing stream.
/// Provided to newly created [`Initiator`](stream::handler::Initiator) structs to
/// carry application context, connection, and stream objects.
#[allow(type_alias_bounds)]
pub type Context<T: Builder> = stream::Context<T, <T::Opener as stream::Opener>::Output>;

/// A builder for creating handlers which implement the [`Initiator`](stream::handler::Initiator) trait.
/// Implementors must create at least 1 builder per initiator type
/// (which may be shared with the [`Receiver`](stream::handler::Receiver) type).
pub trait Builder: stream::Identifier {
	/// The kind of stream that this builder should create.
	///
	/// The type must implement the [`StreamInitiator`](stream::handler::Initiator) trait.
	/// This allows the type of the stream to be known at compile time while also
	/// specializing the function called on the connection which creates the stream.
	type Opener: stream::Opener;
}
