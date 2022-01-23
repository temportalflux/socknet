use crate::{
	stream::{self, processor::ArcProcessor, Stream},
	utility::JoinHandleList,
};
use std::{net::SocketAddr, sync::Arc};

pub type Sender = async_channel::Sender<Connection>;
pub type Receiver = async_channel::Receiver<Connection>;

pub struct Connection {
	connection: quinn::Connection,
	#[allow(dead_code)]
	handles: Arc<JoinHandleList>,
}

impl Connection {
	pub fn remote_address(&self) -> SocketAddr {
		self.connection.remote_address()
	}

	pub fn peer_identity(&self) -> Option<Box<dyn std::any::Any>> {
		self.connection.peer_identity()
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		log::info!(target: crate::LOG, "Closing connection to {}", self.remote_address());
	}
}

impl Connection {
	pub(crate) fn create(
		connection: quinn::NewConnection,
		stream_processor: ArcProcessor,
		error_sender: stream::error::Sender,
	) -> Self {
		let address = connection.connection.remote_address();
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		Self::spawn_stream_handler(
			address,
			stream::Kind::Unidirectional,
			connection.uni_streams,
			stream_processor.clone(),
			error_sender.clone(),
			handles.clone(),
		);
		Self::spawn_stream_handler(
			address,
			stream::Kind::Bidirectional,
			connection.bi_streams,
			stream_processor.clone(),
			error_sender.clone(),
			handles.clone(),
		);
		Self::spawn_stream_handler(
			address,
			stream::Kind::Datagram,
			connection.datagrams,
			stream_processor.clone(),
			error_sender.clone(),
			handles.clone(),
		);

		Self {
			connection: connection.connection,
			handles,
		}
	}

	fn spawn_stream_handler<T, TStream>(
		address: SocketAddr,
		kind: stream::Kind,
		mut incoming: T,
		stream_processor: ArcProcessor,
		error_sender: stream::error::Sender,
		owned_handles: Arc<JoinHandleList>,
	) where
		T: 'static
			+ futures_util::stream::Stream<Item = Result<TStream, quinn::ConnectionError>>
			+ Send
			+ Sync
			+ std::marker::Unpin,
		TStream: 'static + Into<stream::Typed> + Send + Sync,
	{
		let async_owned_handles = owned_handles.clone();
		owned_handles.spawn(async move {
			use futures_util::StreamExt;
			while let Some(status) = incoming.next().await {
				match status {
					Ok(item) => {
						stream_processor.clone().process(
							Stream {
								address: address,
								typed: item.into(),
							},
							async_owned_handles.clone(),
						);
					}
					Err(error) => {
						if let Err(send_err) = error_sender
							.send(stream::error::Error {
								address,
								kind,
								error,
							})
							.await
						{
							log::error!(
								target: crate::LOG,
								"Failed to send stream error, {}.",
								send_err
							);
						}
					}
				}
			}
			Ok(())
		});
	}
}
