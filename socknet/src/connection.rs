use crate::{
	endpoint::Endpoint,
	stream::{self},
	utility::JoinHandleList,
};
use std::{
	net::SocketAddr,
	sync::{Arc, Weak},
};

pub type Sender = async_channel::Sender<Event>;
pub type Receiver = async_channel::Receiver<Event>;

pub enum Event {
	Created(Weak<Connection>),
	Dropped(SocketAddr),
}
impl std::fmt::Debug for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match &self {
			Self::Created(connection) => write!(
				f,
				"Create({})",
				connection.upgrade().unwrap().remote_address()
			),
			Self::Dropped(address) => write!(f, "Dropped({})", address),
		}
	}
}

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
		let identity = self.peer_identity().ok_or(Error::NoIdentity)?;
		let certificates = identity
			.downcast::<Vec<rustls::Certificate>>()
			.map_err(|_| Error::IdentityIsNotCertificate)?;
		Ok(*certificates)
	}

	pub fn certificate(&self) -> anyhow::Result<rustls::Certificate> {
		let mut certificates = self.certificates()?;
		Ok(certificates
			.pop()
			.ok_or(Error::CertificateIdentityIsEmpty)?)
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
		Ok(weak.upgrade().ok_or(Error::ConnectionDropped)?)
	}

	pub fn spawn<T>(self: &Arc<Self>, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		let log_target = self.log_target();
		self.handles.push(tokio::task::spawn(async move {
			if let Err(err) = future.await {
				log::error!(target: &log_target, "{:?}", err);
			}
		}));
	}

	pub async fn open_uni(self: &Arc<Self>) -> anyhow::Result<stream::kind::Send> {
		let send = self.connection.open_uni().await?;
		Ok(send.into())
	}

	pub async fn open_bi(
		self: &Arc<Self>,
	) -> anyhow::Result<(stream::kind::Send, stream::kind::Recv)> {
		let (send, recv) = self.connection.open_bi().await?;
		Ok((send.into(), recv.into()))
	}

	pub fn send_datagram(&self, data: bytes::Bytes) -> anyhow::Result<()> {
		self.connection.send_datagram(data)?;
		Ok(())
	}

	pub fn close(&self, code: u32, reason: &[u8]) {
		self.connection.close(code.into(), reason);
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		log::info!(
			target: &self.log_target(),
			"Closing connection to {}",
			self.remote_address()
		);
		self.endpoint()
			.unwrap()
			.send_connection_event(Event::Dropped(self.remote_address()));
	}
}

impl Connection {
	pub fn log_target(&self) -> String {
		format!("{}/connection[{}]", crate::LOG, self.remote_address())
	}

	pub fn registry(&self) -> anyhow::Result<Arc<stream::Registry>> {
		Ok(self.endpoint()?.stream_registry.clone())
	}

	pub(crate) fn create(endpoint: &Arc<Endpoint>, new_conn: quinn::NewConnection) -> Weak<Self> {
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		let connection = Arc::new(Self {
			endpoint: Arc::downgrade(&endpoint),
			connection: new_conn.connection,
			handles,
		});

		connection
			.clone()
			.spawn_stream_handler("Unidirectional", new_conn.uni_streams);
		connection
			.clone()
			.spawn_stream_handler("Bidirectional", new_conn.bi_streams);
		connection
			.clone()
			.spawn_stream_handler("Datagram", new_conn.datagrams);

		let connection = Arc::downgrade(&connection);
		endpoint.send_connection_event(Event::Created(connection.clone()));
		connection
	}

	fn spawn_stream_handler<T, TStream>(self: Arc<Self>, kind: &'static str, mut incoming: T)
	where
		T: 'static
			+ futures_util::stream::Stream<Item = Result<TStream, quinn::ConnectionError>>
			+ Send
			+ Sync
			+ std::marker::Unpin,
		TStream: 'static + Into<stream::kind::Kind> + Send + Sync,
	{
		let log_target = format!("{}[{} streams]", self.log_target(), kind);
		crate::utility::spawn(log_target.clone(), async move {
			use futures_util::StreamExt;
			while let Some(status) = incoming.next().await {
				match status {
					Ok(item) => {
						let registry = self.endpoint()?.stream_registry.clone();
						registry.create_receiver(self.clone(), item.into());
					}
					Err(error) => {
						log::error!(target: &log_target, "Connection Error: {:?}", error);
						break;
					}
				}
			}
			log::info!(target: &log_target, "Finished receiving {} streams", kind);
			Ok(())
		});
	}
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Cannot get Connection, it has been dropped already.")]
	ConnectionDropped,
	#[error("Connection has no identity.")]
	NoIdentity,
	#[error("Connection's identity is not a list of certificates.")]
	IdentityIsNotCertificate,
	#[error("Connection's identity certificate list is empty.")]
	CertificateIdentityIsEmpty,
}
