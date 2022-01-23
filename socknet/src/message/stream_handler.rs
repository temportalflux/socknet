use crate::{
	stream::{processor::handler::Handler, Stream},
	utility::JoinHandleList,
};
use std::sync::Arc;

#[derive(Default)]
pub struct StreamHandler;

impl Handler for StreamHandler {
	fn process(&self, _stream: Stream, _handle_owner: Arc<JoinHandleList>) {
		log::debug!("handled by message stream handler");
	}
}
