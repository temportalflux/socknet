//! TODO DELETE ME

use crate::stream;
use std::sync::Arc;

mod uni {

	use super::*;

	pub struct Identifier {
		send: Arc<send::AppContext>,
		recv: Arc<recv::AppContext>,
	}

	impl stream::Identifier for Identifier {
		type SendBuilder = send::AppContext;
		type RecvBuilder = recv::AppContext;
		fn unique_id() -> &'static str {
			"uni"
		}
		fn send_builder(&self) -> &Arc<Self::SendBuilder> {
			&self.send
		}
		fn recv_builder(&self) -> &Arc<Self::RecvBuilder> {
			&self.recv
		}
	}

	mod send {
		use super::*;

		pub struct AppContext;
		impl stream::send::AppContext for AppContext {
			type Opener = stream::uni::Opener;
		}

		struct Handler {}
		impl stream::handler::Initiator for Handler {
			type Identifier = Identifier;
		}
		impl From<stream::send::Context<AppContext>> for Handler {
			fn from(_context: stream::send::Context<AppContext>) -> Self {
				Self {}
			}
		}
	}

	mod recv {
		use super::*;

		pub struct AppContext;
		impl stream::recv::AppContext for AppContext {
			type Extractor = stream::uni::Extractor;
			type Receiver = Handler;
		}

		pub struct Handler;
		impl From<stream::recv::Context<AppContext>> for Handler {
			fn from(_context: stream::recv::Context<AppContext>) -> Self {
				Self {}
			}
		}
		impl stream::handler::Receiver for Handler {
			type Identifier = Identifier;
			fn receive(self) {}
		}
	}
}

mod bi {
	use crate::stream::kind::Bidirectional;

	use super::*;

	struct Identifier {
		context: Arc<AppContext>,
	}

	impl stream::Identifier for Identifier {
		type SendBuilder = AppContext;
		type RecvBuilder = AppContext;
		fn unique_id() -> &'static str {
			"bi"
		}
		fn send_builder(&self) -> &Arc<Self::SendBuilder> {
			&self.context
		}
		fn recv_builder(&self) -> &Arc<Self::RecvBuilder> {
			&self.context
		}
	}

	struct AppContext;
	impl stream::send::AppContext for AppContext {
		type Opener = stream::bi::Opener;
	}
	impl stream::recv::AppContext for AppContext {
		type Extractor = stream::bi::Extractor;
		type Receiver = Handler;
	}

	struct Handler {}
	impl stream::handler::Initiator for Handler {
		type Identifier = Identifier;
	}
	impl From<stream::Context<AppContext, Bidirectional>> for Handler {
		fn from(_context: stream::Context<AppContext, Bidirectional>) -> Self {
			Self {}
		}
	}
	impl stream::handler::Receiver for Handler {
		type Identifier = Identifier;
		fn receive(self) {}
	}
}

mod datagram {
	use super::*;

	pub struct Identifier {
		send: Arc<send::AppContext>,
		recv: Arc<recv::AppContext>,
	}

	impl stream::Identifier for Identifier {
		type SendBuilder = send::AppContext;
		type RecvBuilder = recv::AppContext;
		fn unique_id() -> &'static str {
			"datagram"
		}
		fn send_builder(&self) -> &Arc<Self::SendBuilder> {
			&self.send
		}
		fn recv_builder(&self) -> &Arc<Self::RecvBuilder> {
			&self.recv
		}
	}

	mod send {
		use super::*;

		pub struct AppContext;
		impl stream::send::AppContext for AppContext {
			type Opener = stream::datagram::Opener;
		}

		struct Handler;
		impl From<stream::send::Context<AppContext>> for Handler {
			fn from(_context: stream::send::Context<AppContext>) -> Self {
				Self {}
			}
		}
		impl stream::handler::Initiator for Handler {
			type Identifier = Identifier;
		}
	}

	mod recv {
		use super::*;

		pub struct AppContext;
		impl stream::recv::AppContext for AppContext {
			type Extractor = stream::datagram::Extractor;
			type Receiver = Handler;
		}

		pub struct Handler;
		impl From<stream::recv::Context<AppContext>> for Handler {
			fn from(_context: stream::recv::Context<AppContext>) -> Self {
				Self {}
			}
		}
		impl stream::handler::Receiver for Handler {
			type Identifier = Identifier;
			fn receive(self) {}
		}
	}
}
