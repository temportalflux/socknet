use crate::{connection::Connection, stream, utility::PinFutureResult};
use std::sync::Arc;

/// Creates a datagram writer for some connection, resulting in a
/// [`buffer`](stream::kind::SendBytes) which,
/// when [`finish`](stream::kind::Write::finish) is called,
/// transmits the buffered data to the peer.
///
/// The incoming buffer can be extracted using the [`Datagram Extractor`](Extractor).
pub struct Opener;
impl stream::Opener for Opener {
	type Output = stream::kind::SendBytes;
	fn open(connection: Arc<Connection>) -> PinFutureResult<Self::Output> {
		Box::pin(async move { Ok(stream::kind::SendBytes(Vec::new(), connection)) })
	}
}

/// Parses the incoming [`buffer`](stream::kind::RecvBytes),
/// so it can be used by a [`Receiver`](stream::handler::Receiver).
pub struct Extractor;
impl stream::Extractor for Extractor {
	type Output = stream::kind::RecvBytes;
	fn extract(stream: stream::kind::Kind) -> anyhow::Result<Self::Output> {
		match stream {
			stream::kind::Kind::Datagram(bytes) => Ok(bytes),
			_ => unimplemented!(),
		}
	}
}
