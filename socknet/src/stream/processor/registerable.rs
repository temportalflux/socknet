use crate::{connection::Connection, stream};
use std::sync::Weak;

pub trait Registerable {
	fn unique_id() -> &'static str;
	fn create_receiver(connection: Weak<Connection>, stream: stream::Typed) -> anyhow::Result<()>;
}
