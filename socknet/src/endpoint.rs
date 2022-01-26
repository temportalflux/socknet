use crate::{
	connection::{self, Connection},
	stream::Registry,
	utility::JoinHandleList,
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
	pub(crate) stream_registry: Arc<Registry>,
}

impl Drop for Endpoint {
	fn drop(&mut self) {
		log::info!(target: crate::LOG, "Closing endpoint {}", self.address());
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
		stream_registry: Arc<Registry>,
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
			stream_registry,
		}
	}

	fn address(&self) -> SocketAddr {
		self.endpoint.local_addr().unwrap()
	}

	fn log_target(&self) -> String {
		format!("{}/endpoint[{}]", crate::LOG, self.address())
	}

	pub fn spawn<T>(&self, future: T)
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
	pub(crate) fn spawn_connection_listener(self: Arc<Self>, incoming: quinn::Incoming) {
		let log_target = self.log_target();
		tokio::task::spawn(async move {
			if let Err(err) = Endpoint::listen_for_connections(&self, incoming).await {
				log::error!(target: &log_target, "{:?}", err);
			}
		});
	}

	async fn listen_for_connections(
		self: &Arc<Self>,
		mut incoming: quinn::Incoming,
	) -> anyhow::Result<()> {
		use futures_util::StreamExt;
		while let Some(conn) = incoming.next().await {
			let connection: quinn::NewConnection = conn.await?;
			Connection::create(&self, connection);
		}
		Ok(())
	}

	pub async fn connect(
		self: &Arc<Self>,
		address: SocketAddr,
		name: String,
	) -> anyhow::Result<Weak<Connection>> {
		log::info!(target: crate::LOG, "Connecting to {} ({})", name, address);
		let async_endpoint = self.endpoint.clone();
		let connection = async_endpoint.connect(address, &name)?.await?;
		Ok(Connection::create(&self, connection))
	}

	pub(crate) fn send_connection_event(&self, event: connection::Event) {
		use async_channel::TrySendError;
		let log_target = self.log_target();
		match self.connection_sender.try_send(event) {
			Ok(_) => {}
			Err(TrySendError::Full(event)) => {
				log::error!(
					target: &log_target,
					"Failed to enqueue connection event {:?}, the connection queue is full.",
					event
				);
			}
			Err(TrySendError::Closed(event)) => {
				log::error!(
					target: &log_target,
					"Failed to enqueue connection event {:?}, the connection queue has been closed.",
					event
				);
			}
		}
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
