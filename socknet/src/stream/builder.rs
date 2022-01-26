use crate::{connection::Connection, stream};
use std::sync::{Arc, Weak};

pub trait Builder {
	fn unique_id() -> &'static str
	where
		Self: Sized;

	fn make_log_target(&self, _kind: stream::Handler, connection: &Arc<Connection>) -> String
	where
		Self: Sized,
	{
		format!("{}[{}]", Self::unique_id(), connection.remote_address())
	}

	fn open(self: Arc<Self>, connection: Weak<Connection>) -> anyhow::Result<()>
	where
		Self: Sized;

	fn build(
		self: Arc<Self>,
		connection: Arc<Connection>,
		stream: stream::Typed,
	) -> anyhow::Result<()>
	where
		Self: Sized;
}

pub trait Buildable {
	type Builder;
	type Stream;
	fn build(
		builder: Arc<Self::Builder>,
		connection: Arc<Connection>,
		stream: Self::Stream,
	) -> Self;
}
