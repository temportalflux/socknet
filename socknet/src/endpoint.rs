use crate::{
	connection,
	stream::{self, processor::ArcProcessor},
	utility::{self, JoinHandleList},
};
use std::{net::SocketAddr, sync::Arc};

pub enum Config {
	Server(quinn::ServerConfig),
	Client(quinn::ClientConfig),
}

pub struct Endpoint {
	endpoint: Arc<quinn::Endpoint>,
	handles: JoinHandleList,
	connection_sender: connection::Sender,
	connection_receiver: connection::Receiver,
	stream_processor: ArcProcessor,
	error_sender: stream::error::Sender,
}

impl Drop for Endpoint {
	fn drop(&mut self) {
		log::info!(target: crate::LOG, "Closing endpoint {}", self.endpoint.local_addr().unwrap());
	}
}

impl Endpoint {
	pub(crate) fn new(
		endpoint: quinn::Endpoint,
		stream_processor: ArcProcessor,
		error_sender: stream::error::Sender,
	) -> Self {
		let endpoint = Arc::new(endpoint);
		let (connection_sender, connection_receiver) = async_channel::unbounded();
		Self {
			endpoint,
			handles: JoinHandleList::new(),
			connection_sender,
			connection_receiver,
			stream_processor,
			error_sender,
		}
	}

	fn spawn_owned<T>(&self, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		self.handles.push(utility::spawn(future));
	}

	pub fn connection_receiver(&self) -> &connection::Receiver {
		&self.connection_receiver
	}
}

impl Endpoint {
	pub(crate) fn listen_for_connections(&mut self, mut incoming: quinn::Incoming) {
		let connection_sender = self.connection_sender.clone();
		let stream_processor = self.stream_processor.clone();
		let error_sender = self.error_sender.clone();
		self.spawn_owned(async move {
			use futures_util::StreamExt;
			while let Some(conn) = incoming.next().await {
				let new_connection: quinn::NewConnection = conn.await?;
				Endpoint::send_connection(
					&connection_sender,
					new_connection,
					stream_processor.clone(),
					error_sender.clone(),
				);
			}
			Ok(())
		});
	}

	pub fn connect(&self, address: SocketAddr, name: String) {
		let async_endpoint = self.endpoint.clone();
		let connection_sender = self.connection_sender.clone();
		let stream_processor = self.stream_processor.clone();
		let error_sender = self.error_sender.clone();
		self.spawn_owned(async move {
			log::info!(target: crate::LOG, "Connecting to {} ({})", name, address);
			let server_conn = async_endpoint.connect(address, &name)?.await?;
			Self::send_connection(
				&connection_sender,
				server_conn,
				stream_processor,
				error_sender,
			);
			Ok(())
		});
	}

	fn send_connection(
		sender: &connection::Sender,
		connection: quinn::NewConnection,
		stream_processor: ArcProcessor,
		error_sender: stream::error::Sender,
	) {
		use async_channel::TrySendError;
		let connection = connection::Connection::create(connection, stream_processor, error_sender);
		match sender.try_send(connection) {
			Ok(_) => {}
			Err(TrySendError::Full(connection)) => {
				log::error!(
					target: crate::LOG,
					"Failed to enqueue new connection from {}, the connection queue is full.",
					connection.remote_address()
				);
			}
			Err(TrySendError::Closed(connection)) => {
				log::error!(
					target: crate::LOG,
					"Failed to enqueue new connection from {}, the connection queue is no longer accepting connections.",
					connection.remote_address()
				);
			}
		}
	}
}
