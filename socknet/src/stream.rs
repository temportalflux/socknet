use crate::connection::Connection;
use std::sync::{Arc, Weak};

mod recv_bytes;
pub use recv_bytes::*;

pub mod error;

mod kind;
pub use kind::*;

pub mod processor;

mod recv;
pub use recv::*;
mod send;
pub use send::*;

mod typed;
pub use typed::*;

pub use crate::utility::JoinHandleList as TaskOwner;
pub use anyhow::Result;

pub trait Initiator<TStream> {
	fn open(connection: &Arc<Connection>) -> Result<()>;
}

pub trait Responder<TStream> {
	fn receive(connection: Weak<Connection>, stream: TStream) -> Result<()>;
}
