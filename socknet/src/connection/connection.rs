use crate::{
	connection::{active::Active, event::Event, opened::Opened, Datagram, Error},
	endpoint::Endpoint,
	utility::JoinHandleList,
};
use crate::{
	stream::{
		self,
		kind::{recv, send},
	},
	utility::PinFutureResultLifetime,
};
use std::{
	net::SocketAddr,
	sync::{Arc, Weak},
};

pub struct Connection {
	pub(crate) endpoint: Weak<Endpoint>,
	pub(crate) connection: Box<dyn Active + Send + Sync + 'static>,
	#[allow(dead_code)]
	pub(crate) handles: Arc<JoinHandleList>,
}

impl Connection {
	pub(crate) fn create<T>(endpoint: &Arc<Endpoint>, opened: T) -> Weak<Self>
	where
		T: Opened,
	{
		let connection = opened.create(Arc::downgrade(&endpoint));
		endpoint.send_connection_event(Event::Created(connection.clone()));
		connection
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

	pub fn registry(&self) -> anyhow::Result<Arc<stream::Registry>> {
		Ok(self.endpoint()?.stream_registry.clone())
	}
}

impl Active for Connection {
	fn remote_address(&self) -> SocketAddr {
		self.connection.remote_address()
	}

	fn peer_identity(&self) -> Option<Box<dyn std::any::Any>> {
		self.connection.peer_identity()
	}

	fn is_local(&self) -> bool {
		self.connection.is_local()
	}

	fn open_uni<'a>(&'a self) -> PinFutureResultLifetime<'a, send::Ongoing> {
		self.connection.open_uni()
	}

	fn open_bi<'a>(&'a self) -> PinFutureResultLifetime<'a, (send::Ongoing, recv::Ongoing)> {
		self.connection.open_bi()
	}

	fn send_datagram(&self, datagram: Datagram) -> anyhow::Result<()> {
		self.connection.send_datagram(datagram)
	}

	fn close(&self, code: u32, reason: &[u8]) {
		self.connection.close(code, reason);
	}
}

impl Connection {
	pub fn log_target(&self) -> String {
		format!("{}/connection[{}]", crate::LOG, self.remote_address())
	}

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

	pub(crate) fn spawn_stream_handler<T, TStream>(
		self: Arc<Self>,
		kind: &'static str,
		mut incoming: T,
	) where
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
			use quinn::{ApplicationClose, ConnectionError};
			let mut close_cause = None;
			while let Some(status) = incoming.next().await {
				match status {
					Ok(item) => {
						let registry = self.endpoint()?.stream_registry.clone();
						registry.create_receiver(self.clone(), item.into());
					}
					Err(error) => {
						close_cause = Some(error);
						break;
					}
				}
			}

			let cause = match close_cause {
				Some(error) => {
					if let ConnectionError::ApplicationClosed(ApplicationClose {
						error_code,
						reason,
					}) = &error
					{
						format!("by client with exit code {}", error_code)
					} else {
						format!("due to error: {:?}", error)
					}
				}
				None => "naturally".to_owned(),
			};

			log::trace!(target: &log_target, "Incoming stream closed {}", cause);

			Ok(())
		});
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		log::info!(
			target: &self.log_target(),
			"Closing connection to {}",
			self.remote_address()
		);
		if let Ok(endpoint) = self.endpoint() {
			endpoint.send_connection_event(Event::Dropped(self.remote_address()));
		}
	}
}

#[repr(u32)]
enum ErrorCode {
	Natural = 0,
}
