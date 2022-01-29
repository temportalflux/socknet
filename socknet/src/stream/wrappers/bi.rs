use crate::{connection::Connection, stream, utility::PinFutureResult};
use std::sync::Arc;

/// Initates/opens a bidirectional outgoing stream for some connection,
/// resulting in outgoing [`send`](stream::kind::Send) & [`recv`](stream::kind::Recv) streams.
///
/// The incoming stream can be extracted using the [`Bidirectional Extractor`](Extractor).
///
/// Calls [`open_bi`](Connection::open_bi) under the hood.
pub struct Opener;
impl stream::Opener for Opener {
	type Output = (stream::kind::Send, stream::kind::Recv);
	fn open(connection: Arc<Connection>) -> PinFutureResult<Self::Output> {
		Box::pin(async move { connection.open_bi().await })
	}
}

/// Parses the incoming [`send`](stream::kind::Send) & [`recv`](stream::kind::Recv) streams,
/// so they can be used by a [`Receiver`](stream::handler::Receiver).
pub struct Extractor;
impl stream::Extractor for Extractor {
	type Output = (stream::kind::Send, stream::kind::Recv);
	fn extract(stream: stream::kind::Kind) -> anyhow::Result<Self::Output> {
		match stream {
			stream::kind::Kind::Bidirectional(send, recv) => Ok((send, recv)),
			_ => unimplemented!(),
		}
	}
}
