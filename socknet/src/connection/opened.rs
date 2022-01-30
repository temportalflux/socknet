use crate::{connection::Connection, endpoint::Endpoint};
use std::sync::Weak;

mod remote;
pub use remote::*;

mod local;
pub use local::*;

pub trait Opened {
	fn create(self, endpoint: Weak<Endpoint>) -> Weak<Connection>;
}
