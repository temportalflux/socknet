use crate::{stream::Stream, utility::JoinHandleList};
use std::sync::Arc;

pub trait Handler {
	fn process(&self, stream: Stream, handle_owner: Arc<JoinHandleList>);
}
