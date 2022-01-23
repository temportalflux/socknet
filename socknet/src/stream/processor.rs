use crate::{stream::Stream, utility::JoinHandleList};
use std::sync::Arc;

pub type ArcProcessor = Arc<dyn Processor + Send + Sync + 'static>;

pub trait Processor {
	fn process(self: Arc<Self>, stream: Stream, handle_owner: Arc<JoinHandleList>);
}

pub mod handler;
