use crate::{
	connection::{active, opened::Opened, Connection},
	endpoint::Endpoint,
	utility::JoinHandleList,
};
use std::sync::{Arc, Weak};

pub struct Remote(quinn::NewConnection);

impl From<quinn::NewConnection> for Remote {
	fn from(other: quinn::NewConnection) -> Self {
		Self(other)
	}
}

impl Opened for Remote {
	fn create(self, endpoint: Weak<Endpoint>) -> Weak<Connection> {
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		let connection = Arc::new(Connection {
			endpoint,
			connection: Box::new(active::Remote(self.0.connection)),
			handles,
		});

		connection
			.clone()
			.spawn_stream_handler("Unidirectional", self.0.uni_streams);
		connection
			.clone()
			.spawn_stream_handler("Bidirectional", self.0.bi_streams);
		connection
			.clone()
			.spawn_stream_handler("Datagram", self.0.datagrams);

		Arc::downgrade(&connection)
	}
}
