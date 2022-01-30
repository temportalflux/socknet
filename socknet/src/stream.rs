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
	/// Returns a static identifier that should be unique
	/// when compared against all stream handlers in a given application.
	/// An acceptable approach could be the implementing struct's module path,
	/// though the longer this id is, the more bytes will be written to an outgoing stream.
	fn unique_id() -> &'static str
	where
		Self: Sized;
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
