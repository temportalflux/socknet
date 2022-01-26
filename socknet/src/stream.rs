mod kind;
pub use kind::*;
mod typed;
pub use typed::*;

mod builder;
pub use builder::*;
mod registry;
pub use registry::*;

mod send;
pub use send::*;
mod recv;
pub use recv::*;
mod recv_bytes;
pub use recv_bytes::*;

pub use crate::utility::{spawn, JoinHandleList as TaskOwner};
pub use anyhow::Result;

pub enum Handler {
	Initiator,
	Responder,
}
impl std::fmt::Display for Handler {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Initiator => write!(f, "Initiator"),
			Self::Responder => write!(f, "Responder"),
		}
	}
}
