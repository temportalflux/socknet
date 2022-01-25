use crate::{
	connection::{self, Connection},
	stream::{self, processor::ArcProcessor},
	utility::{self, JoinHandleList},
};
use std::{
	net::SocketAddr,
	sync::{Arc, Weak},
};

pub enum Config {
	Server(ServerConfig),
	Client(ClientConfig),
}

pub struct ServerConfig {
	pub core: quinn::ServerConfig,
	pub certificate: rustls::Certificate,
	pub private_key: rustls::PrivateKey,
}

pub struct ClientConfig {
	pub core: quinn::ClientConfig,
	pub certificate: rustls::Certificate,
	pub private_key: rustls::PrivateKey,
}

pub struct Endpoint {
	endpoint: Arc<quinn::Endpoint>,
	certificate: rustls::Certificate,
	private_key: rustls::PrivateKey,
	handles: JoinHandleList,
	pub(crate) connection_sender: connection::Sender,
	connection_receiver: connection::Receiver,
	pub(crate) stream_processor: ArcProcessor,
	pub(crate) error_sender: stream::error::Sender,
}

impl Drop for Endpoint {
	fn drop(&mut self) {
		log::info!(
			target: crate::LOG,
			"Closing endpoint {}",
			self.endpoint.local_addr().unwrap()
		);
	}
}

impl Endpoint {
	pub fn upgrade(weak: &Weak<Self>) -> anyhow::Result<Arc<Self>> {
		Ok(weak.upgrade().ok_or(EndpointDropped)?)
	}

	pub(crate) fn new(
		endpoint: quinn::Endpoint,
		certificate: rustls::Certificate,
		private_key: rustls::PrivateKey,
		stream_processor: ArcProcessor,
		error_sender: stream::error::Sender,
	) -> Self {
		let endpoint = Arc::new(endpoint);
		let (connection_sender, connection_receiver) = async_channel::unbounded();
		Self {
			endpoint,
			certificate,
			private_key,
			handles: JoinHandleList::new(),
			connection_sender,
			connection_receiver,
			stream_processor,
			error_sender,
		}
	}

	pub fn spawn_owned<T>(&self, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		self.handles.push(utility::spawn(future));
	}

	pub fn connection_receiver(&self) -> &connection::Receiver {
		&self.connection_receiver
	}

	pub fn certificate(&self) -> &rustls::Certificate {
		&self.certificate
	}

	pub fn private_key(&self) -> &rustls::PrivateKey {
		&self.private_key
	}

	pub fn fingerprint(&self) -> String {
		crate::utility::fingerprint(&self.certificate)
	}
}

impl Endpoint {
	pub(crate) fn listen_for_connections(self: &Arc<Self>, mut incoming: quinn::Incoming) {
		let weak_endpoint = Arc::downgrade(&self);
		self.spawn_owned(async move {
			use futures_util::StreamExt;
			while let Some(conn) = incoming.next().await {
				let connection: quinn::NewConnection = conn.await?;
				let arc_endpoint = weak_endpoint.upgrade().ok_or(EndpointDropped)?;
				Connection::create(arc_endpoint, connection);
			}
			Ok(())
		});
	}

	pub async fn connect(
		self: &Arc<Self>,
		address: SocketAddr,
		name: String,
	) -> anyhow::Result<Arc<Connection>> {
		log::info!(target: crate::LOG, "Connecting to {} ({})", name, address);
		let async_endpoint = self.endpoint.clone();
		let connection = async_endpoint.connect(address, &name)?.await?;
		let connection = Connection::create(self.clone(), connection);
		Ok(connection)
	}
}

pub struct EndpointDropped;
impl std::error::Error for EndpointDropped {}
impl std::fmt::Debug for EndpointDropped {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		<Self as std::fmt::Display>::fmt(&self, f)
	}
}
impl std::fmt::Display for EndpointDropped {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Cannot get Endpoint, it has been dropped already.",)
	}
}
