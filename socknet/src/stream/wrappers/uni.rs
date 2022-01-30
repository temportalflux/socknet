use crate::{
	connection::{active::Active, Connection},
	stream,
	utility::PinFutureResult,
};
use std::sync::Arc;

/// Initates/opens a unidirectional outgoing stream for some connection,
/// resulting in an outgoing [`send stream`](stream::kind::Send).
///
/// The incoming stream can be extracted using the [`Unidirectional Extractor`](Extractor).
///
/// Calls [`open_uni`](Connection::open_uni) under the hood.
pub struct Opener;
impl stream::Opener for Opener {
	type Output = stream::kind::send::Ongoing;
	fn open(connection: Arc<Connection>) -> PinFutureResult<Self::Output> {
		Box::pin(async move { connection.open_uni().await })
	}
}

/// Parses the incoming [`recv stream`](stream::kind::Recv)
/// so it can be used by a [`Receiver`](stream::handler::Receiver).
pub struct Extractor;
impl stream::Extractor for Extractor {
	type Output = stream::kind::recv::Ongoing;
	fn extract(stream: stream::kind::Kind) -> anyhow::Result<Self::Output> {
		match stream {
			stream::kind::Kind::Unidirectional(recv) => Ok(recv),
			_ => unimplemented!(),
		}
	}
}
