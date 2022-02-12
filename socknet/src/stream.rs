use crate::{connection::Connection, utility::PinFutureResult};
use std::sync::Arc;

/// Contains structures for the different forms of data streams.
pub mod kind;
pub(crate) mod local;

/// API interface for creating builders for [`Receivers`](handler::Receiver) and their contexts.
pub mod recv;
/// API interface for creating builders for [`Initiators`](handler::Initiator) and their contexts.
pub mod send;

#[doc(hidden)]
mod wrappers;
pub use wrappers::*;

pub mod example;

/// Traits used to implement stream initiation and reception.
pub mod handler;

#[doc(hidden)]
mod registry;
pub use registry::*;

/// Identifies a stream handler.
/// Used to write the id to an outgoing/send stream so when the
/// incoming/receiving end finds the stream, it knows what handler to build to respond/react.
pub trait Identifier {
	type SendBuilder: send::AppContext;
	type RecvBuilder: recv::AppContext;

	/// Returns a static identifier that should be unique
	/// when compared against all stream handlers in a given application.
	/// An acceptable approach could be the implementing struct's module path,
	/// though the longer this id is, the more bytes will be written to an outgoing stream.
	fn unique_id() -> &'static str
	where
		Self: Sized;

	fn send_builder(&self) -> &Arc<Self::SendBuilder>;
	fn recv_builder(&self) -> &Arc<Self::RecvBuilder>;

	fn log_category(prefix: &str, connection: &Arc<Connection>) -> String
	where
		Self: Sized,
	{
		use crate::connection::Active;
		format!(
			"{}/{}[{}]",
			prefix,
			Self::unique_id(),
			connection.remote_address()
		)
	}

	fn open(
		connection: Arc<Connection>,
	) -> PinFutureResult<<<Self::SendBuilder as send::AppContext>::Opener as Opener>::Output> {
		<Self::SendBuilder as send::AppContext>::open(connection)
	}

	fn open_context(
		&self,
		connection: Arc<Connection>,
	) -> PinFutureResult<send::Context<Self::SendBuilder>>
	where
		Self: Sized + Send + Sync + 'static,
		Self::SendBuilder: send::AppContext + Send + Sync + 'static,
		<<Self::SendBuilder as send::AppContext>::Opener as Opener>::Output:
			kind::send::Write + Send,
	{
		let builder = self.send_builder().clone();
		Box::pin(async move {
			use send::AppContext;
			let mut stream = Self::SendBuilder::open(connection.clone()).await?;
			// Because the stream is identified, we should always write the id of the stream when its opened.
			// If a user is not using the built-in identifier system, they shouldn't be using this trait.
			{
				use kind::send::Write;
				stream.write(&Self::unique_id().to_owned()).await?;
			}
			Ok(send::Context {
				builder,
				connection,
				stream,
			})
		})
	}
}

/// Generic context object specialized by the [`send`](send::Context) and [`recv`](recv::Context) modules.
pub struct Context<T, S> {
	/// The builder used to create some stream handler.
	pub builder: Arc<T>,
	/// The connection this context pertains to.
	pub connection: Arc<Connection>,
	/// The stream(s) for the context's connections that the handler needs to process.
	pub stream: S,
}

/// Trait implemented to specialize how different streams are opened.
///
/// Built-In Implementations:
/// - [`Unidirectional`](uni::Opener)
/// - [`Bidirectional`](bi::Opener)
/// - [`Datagram`](datagram::Opener)
pub trait Opener {
	type Output;
	fn open(connection: Arc<Connection>) -> PinFutureResult<Self::Output>;
}

/// Trait implemented to specialize how different streams are extracted from the [`Kind`](kind::Kind) enum.
///
/// Built-In Implementations:
/// - [`Unidirectional`](uni::Extractor)
/// - [`Bidirectional`](bi::Extractor)
/// - [`Datagram`](datagram::Extractor)
pub trait Extractor {
	type Output;
	fn extract(stream: kind::Kind) -> anyhow::Result<Self::Output>;
}
