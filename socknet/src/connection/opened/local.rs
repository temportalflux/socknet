use crate::{
	connection::{active, opened::Opened, Connection},
	endpoint::Endpoint,
	stream::{
		kind::{
			recv::ongoing::local::Internal as RecvLocalOngoing,
			send::ongoing::local::Internal as SendLocalOngoing,
		},
		local::{AnyBox, Incoming},
	},
	utility::JoinHandleList,
};
use std::sync::{Arc, Weak};

pub struct Local {
	active: active::Local,
	incoming_uni_streams: Incoming<RecvLocalOngoing>,
	incoming_bi_streams: Incoming<(SendLocalOngoing, RecvLocalOngoing)>,
	incoming_datagrams: Incoming<Vec<AnyBox>>,
}

impl Local {
	pub fn new(endpoint: Weak<Endpoint>) -> Self {
		let (outgoing_uni, incoming_uni_streams) = async_channel::unbounded();
		let (outgoing_bi, incoming_bi_streams) = async_channel::unbounded();
		let (outgoing_data, incoming_datagrams) = async_channel::unbounded();
		Self {
			active: active::Local {
				endpoint,
				uni_streams: outgoing_uni,
				bi_streams: outgoing_bi,
				datagrams: outgoing_data,
			},
			incoming_uni_streams,
			incoming_bi_streams,
			incoming_datagrams,
		}
	}
}

impl Opened for Local {
	fn create(self, endpoint: Weak<Endpoint>) -> Weak<Connection> {
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		let connection = Arc::new(Connection {
			endpoint,
			connection: Box::new(self.active),
			handles,
		});

		connection
			.clone()
			.spawn_stream_handler("Unidirectional", self.incoming_uni_streams);
		connection
			.clone()
			.spawn_stream_handler("Bidirectional", self.incoming_bi_streams);
		connection
			.clone()
			.spawn_stream_handler("Datagram", self.incoming_datagrams);

		Arc::downgrade(&connection)
	}
}
