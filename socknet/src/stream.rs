use crate::connection::Connection;
use std::sync::{Arc, Weak};

mod recv_bytes;
pub use recv_bytes::*;

mod kind;
pub use kind::*;

pub mod processor;

mod recv;
pub use recv::*;
mod send;
pub use send::*;

mod typed;
pub use typed::*;

pub use crate::utility::{spawn, JoinHandleList as TaskOwner};
pub use anyhow::Result;

pub enum Handler {
	Initiator,
	Responder,
}

pub trait LogSource {
	fn target(kind: Handler, connection: &Arc<Connection>) -> String;
}

pub trait Initiator<TStream>: LogSource {
	fn open(connection: &Weak<Connection>) -> Result<()>;
}

pub trait Responder<TStream>: LogSource {
	fn receive(connection: Arc<Connection>, stream: TStream) -> Result<()>;
}
