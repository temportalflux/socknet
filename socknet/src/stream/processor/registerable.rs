use crate::{connection::Connection, stream};
use std::sync::Arc;

pub trait Registerable {
	fn unique_id() -> &'static str;
	fn create_receiver(connection: Arc<Connection>, stream: stream::Typed) -> anyhow::Result<()>;
}
