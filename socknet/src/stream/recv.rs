use crate::{connection::Connection, stream};
use std::sync::Arc;

/// Contextual information about an incoming stream.
/// Provided to newly created [`Receiver`](stream::handler::Receiver) structs to
/// carry application context, connection, and stream objects.
#[allow(type_alias_bounds)]
pub type Context<T: AppContext> = stream::Context<T, <T::Extractor as stream::Extractor>::Output>;

/// A builder for creating handlers which implement the [`Receiver`](stream::handler::Receiver) trait.
/// Implementors must create at least 1 builder per receiver type
/// (which may be shared with the [`Initiator`](stream::handler::Initiator) type).
pub trait AppContext {
	/// The kind of stream that this builder should create.
	///
	/// The type must implement the [`StreamExtractor`](stream::Extractor) trait.
	/// This allows the type of the stream to be known at compile time while also
	/// specializing what type of stream the receiver gets.
	type Extractor: stream::Extractor;

	/// The [`Receiver`](stream::handler::Receiver) type that this builder creates
	/// when a stream with its [`unique_id`](stream::Identifier::unique_id) is encountered.
	type Receiver: stream::handler::Receiver;

	/// Extracts the stream using the [`Extractor`](Self::Extractor) associated type,
	/// then wraps the context, connection, and extracted stream into a [`context`](stream::recv::Context).
	fn into_context(
		self: Arc<Self>,
		connection: Arc<Connection>,
		stream: stream::kind::Kind,
	) -> anyhow::Result<Context<Self>>
	where
		Self: Sized,
	{
		let stream = <Self::Extractor as stream::Extractor>::extract(stream)?;
		Ok(Context {
			builder: self,
			connection,
			stream,
		})
	}

	/// Takes the context created by [`into_context`](Self::into_context),
	/// creates the [`Receiver`](Self::Receiver) object from the context,
	/// and then calls [`receive`](stream::handler::Receiver::receive) to start parsing/handling the incoming stream.
	fn process(context: Context<Self>)
	where
		Self: Sized,
		Self::Receiver: From<Context<Self>> + stream::handler::Receiver,
	{
		use stream::handler::Receiver;
		Self::Receiver::from(context).receive();
	}
}
