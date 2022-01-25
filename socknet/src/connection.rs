use crate::{
	endpoint::Endpoint,
	stream::{self},
	utility::JoinHandleList,
};
use std::{
	net::SocketAddr,
	sync::{Arc, Weak},
};

pub type Sender = async_channel::Sender<Arc<Connection>>;
pub type Receiver = async_channel::Receiver<Arc<Connection>>;

pub struct Connection {
	endpoint: Weak<Endpoint>,
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

	pub fn certificates(&self) -> anyhow::Result<Vec<rustls::Certificate>> {
		let identity = self.peer_identity().ok_or(NoIdentity)?;
		let certificates = identity
			.downcast::<Vec<rustls::Certificate>>()
			.map_err(|_| IdentityIsNotCertificate)?;
		Ok(*certificates)
	}

	pub fn certificate(&self) -> anyhow::Result<rustls::Certificate> {
		let mut certificates = self.certificates()?;
		Ok(certificates.pop().ok_or(CertificateIdentityIsEmpty)?)
	}

	pub fn fingerprint(&self) -> anyhow::Result<String> {
		let certificate = self.certificate()?;
		Ok(crate::utility::fingerprint(&certificate))
	}

	pub fn endpoint(&self) -> anyhow::Result<Arc<Endpoint>> {
		Endpoint::upgrade(&self.endpoint)
	}
}

impl Connection {
	pub fn upgrade(weak: &Weak<Self>) -> anyhow::Result<Arc<Self>> {
		Ok(weak.upgrade().ok_or(ConnectionDropped)?)
	}

	pub fn spawn<T>(&self, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		self.handles.spawn(future);
	}

	pub async fn open_uni(self: &Arc<Self>) -> anyhow::Result<stream::Send> {
		let send = self.connection.open_uni().await?;
		Ok(stream::Send(send))
	}

	pub async fn open_bi(self: &Arc<Self>) -> anyhow::Result<(stream::Send, stream::Recv)> {
		let (send, recv) = self.connection.open_bi().await?;
		Ok((stream::Send(send), stream::Recv(recv)))
	}

	pub fn send_datagram(&self, data: bytes::Bytes) -> anyhow::Result<()> {
		self.connection.send_datagram(data)?;
		Ok(())
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		log::info!(
			target: crate::LOG,
			"Closing connection to {}",
			self.remote_address()
		);
	}
}

impl Connection {
	pub(crate) fn create(endpoint: Arc<Endpoint>, new_conn: quinn::NewConnection) -> Arc<Self> {
		use async_channel::TrySendError;
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		let connection = Arc::new(Self {
			endpoint: Arc::downgrade(&endpoint),
			connection: new_conn.connection,
			handles,
		});

		connection
			.clone()
			.spawn_stream_handler(stream::Kind::Unidirectional, new_conn.uni_streams);
		connection
			.clone()
			.spawn_stream_handler(stream::Kind::Bidirectional, new_conn.bi_streams);
		connection
			.clone()
			.spawn_stream_handler(stream::Kind::Datagram, new_conn.datagrams);

		match endpoint.connection_sender.try_send(connection.clone()) {
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

		connection
	}

	fn spawn_stream_handler<T, TStream>(self: Arc<Self>, kind: stream::Kind, mut incoming: T)
	where
		T: 'static
			+ futures_util::stream::Stream<Item = Result<TStream, quinn::ConnectionError>>
			+ Send
			+ Sync
			+ std::marker::Unpin,
		TStream: 'static + Into<stream::Typed> + Send + Sync,
	{
		let weak_conn = Arc::downgrade(&self);
		self.clone().spawn(async move {
			use futures_util::StreamExt;
			while let Some(status) = incoming.next().await {
				let connection = weak_conn.upgrade().ok_or(ConnectionDropped)?;
				let endpoint = connection.endpoint()?;
				match status {
					Ok(item) => {
						endpoint
							.stream_processor
							.clone()
							.create_receiver(connection.clone(), item.into());
					}
					Err(error) => {
						if let Err(send_err) = endpoint
							.error_sender
							.send(stream::error::Error {
								address: connection.remote_address(),
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

pub struct ConnectionDropped;
impl std::error::Error for ConnectionDropped {}
impl std::fmt::Debug for ConnectionDropped {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for ConnectionDropped {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Cannot get Connection, it has been dropped already.",)
	}
}

pub struct NoIdentity;
impl std::error::Error for NoIdentity {}
impl std::fmt::Debug for NoIdentity {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for NoIdentity {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Connection has no identity.",)
	}
}

pub struct IdentityIsNotCertificate;
impl std::error::Error for IdentityIsNotCertificate {}
impl std::fmt::Debug for IdentityIsNotCertificate {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for IdentityIsNotCertificate {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Connection's identity is not a list of certificates.",)
	}
}

pub struct CertificateIdentityIsEmpty;
impl std::error::Error for CertificateIdentityIsEmpty {}
impl std::fmt::Debug for CertificateIdentityIsEmpty {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for CertificateIdentityIsEmpty {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Connection's identity certificate list is empty.",)
	}
}
