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
	connection: Internal,
	#[allow(dead_code)]
	handles: Arc<JoinHandleList>,
}

pub(crate) enum Datagram {
	Serialized(bytes::Bytes),
	Local(Vec<stream::local::AnyBox>),
}
impl From<bytes::Bytes> for Datagram {
	fn from(data: bytes::Bytes) -> Self {
		Self::Serialized(data)
	}
}
impl From<Vec<stream::local::AnyBox>> for Datagram {
	fn from(data: Vec<stream::local::AnyBox>) -> Self {
		Self::Local(data)
	}
}

struct LocalConnection {
	uni_streams: stream::local::Outgoing<stream::kind::recv::ongoing::local::Internal>,
	bi_streams: stream::local::Outgoing<(
		stream::kind::send::ongoing::local::Internal,
		stream::kind::recv::ongoing::local::Internal,
	)>,
	datagrams: stream::local::Outgoing<Vec<stream::local::AnyBox>>,
}
enum Internal {
	Peer(quinn::Connection),
	Local(LocalConnection),
}

impl Connection {
	pub fn remote_address(&self) -> SocketAddr {
		match &self.connection {
			Internal::Peer(connection) => connection.remote_address(),
			Internal::Local(_) => self.endpoint().unwrap().address(),
		}
	}

	pub fn peer_identity(&self) -> Option<Box<dyn std::any::Any>> {
		match &self.connection {
			Internal::Peer(connection) => connection.peer_identity(),
			Internal::Local(_) => Some(Box::new(vec![self
				.endpoint()
				.unwrap()
				.certificate()
				.clone()])),
		}
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

	pub fn is_local(&self) -> bool {
		match &self.connection {
			Internal::Peer(_) => false,
			Internal::Local(_) => true,
		}
	}

	pub async fn open_uni(self: &Arc<Self>) -> anyhow::Result<stream::kind::send::Ongoing> {
		match &self.connection {
			Internal::Peer(connection) => {
				let send = connection.open_uni().await?;
				Ok(send.into())
			}
			Internal::Local(local) => {
				let (send, recv) = async_channel::unbounded::<stream::local::AnyBox>();
				local.uni_streams.send(Ok(recv)).await?;
				Ok(send.into())
			}
		}
	}

	pub async fn open_bi(
		self: &Arc<Self>,
	) -> anyhow::Result<(stream::kind::send::Ongoing, stream::kind::recv::Ongoing)> {
		use stream::kind::{recv, send};
		match &self.connection {
			Internal::Peer(connection) => {
				let (send, recv) = connection.open_bi().await?;
				Ok((
					send::Ongoing::Remote(send.into()),
					recv::Ongoing::Remote(recv.into()),
				))
			}
			Internal::Local(local) => {
				let (a_send, b_recv) = async_channel::unbounded::<stream::local::AnyBox>();
				let (b_send, a_recv) = async_channel::unbounded::<stream::local::AnyBox>();
				local.bi_streams.send(Ok((b_send, b_recv))).await?;
				Ok((
					send::Ongoing::Local(a_send.into()),
					recv::Ongoing::Local(a_recv.into()),
				))
			}
		}
	}

	pub(crate) fn send_datagram(&self, datagram: Datagram) -> anyhow::Result<()> {
		match (&self.connection, datagram) {
			(Internal::Peer(connection), Datagram::Serialized(bytes)) => {
				connection.send_datagram(bytes)?;
				Ok(())
			}
			(Internal::Local(local), Datagram::Local(data)) => {
				local.datagrams.try_send(Ok(data))?;
				Ok(())
			}
			_ => unimplemented!(),
		}
	}

	pub fn close(&self, code: u32, reason: &[u8]) {
		if let Internal::Peer(connection) = &self.connection {
			connection.close(code.into(), reason);
		}
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

	pub(crate) fn create(
		endpoint: &Arc<Endpoint>,
		new_conn: Option<quinn::NewConnection>,
	) -> Weak<Self> {
		let handles = Arc::new(JoinHandleList::with_capacity(3));

		let connection = match new_conn {
			Some(new_conn) => {
				let connection = Arc::new(Self {
					endpoint: Arc::downgrade(&endpoint),
					connection: Internal::Peer(new_conn.connection),
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

				connection
			}
			None => {
				let (send_uni, uni_streams) = async_channel::unbounded();
				let (send_bi, bi_streams) = async_channel::unbounded();
				let (send_data, datagrams) = async_channel::unbounded();

				let connection = Arc::new(Self {
					endpoint: Arc::downgrade(&endpoint),
					connection: Internal::Local(LocalConnection {
						uni_streams: send_uni,
						bi_streams: send_bi,
						datagrams: send_data,
					}),
					handles,
				});

				connection
					.clone()
					.spawn_stream_handler("Unidirectional", uni_streams);
				connection
					.clone()
					.spawn_stream_handler("Bidirectional", bi_streams);
				connection
					.clone()
					.spawn_stream_handler("Datagram", datagrams);

				connection
			}
		};

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
