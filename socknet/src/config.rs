use crate::{
	endpoint::{self, Endpoint},
	stream,
};
use std::{net::SocketAddr, sync::Arc};

pub struct Config {
	pub endpoint: endpoint::Config,
	pub address: SocketAddr,
	pub stream_registry: Arc<stream::Registry>,
}

impl Config {
	pub fn build(self) -> anyhow::Result<Arc<Endpoint>> {
		log::info!(
			target: crate::LOG,
			"Creating network on address({}) identity({})",
			self.address,
			self.endpoint.fingerprint()
		);
		match self.endpoint {
			endpoint::Config::Server(config) => {
				let (endpoint, incoming) = quinn::Endpoint::server(config.core, self.address)?;
				let endpoint = Arc::new(Endpoint::new(
					endpoint,
					config.certificate,
					config.private_key,
					self.stream_registry,
				));
				endpoint.spawn_connection_listener(incoming);
				Ok(endpoint)
			}
			endpoint::Config::Client(config) => {
				let mut endpoint = quinn::Endpoint::client(self.address)?;
				endpoint.set_default_client_config(config.core);
				Ok(Arc::new(Endpoint::new(
					endpoint,
					config.certificate,
					config.private_key,
					self.stream_registry,
				)))
			}
		}
	}
}
