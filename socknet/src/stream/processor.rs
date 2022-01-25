use crate::{connection::Connection, stream::Typed};
use std::sync::Arc;

mod registerable;
pub use registerable::*;
mod registration;
pub(crate) use registration::Registration;
mod registry;
pub use registry::*;

pub type ArcProcessor = Arc<dyn Processor + Send + Sync + 'static>;

pub trait Processor {
	fn create_receiver(self: Arc<Self>, connection: Arc<Connection>, stream: Typed);
}
