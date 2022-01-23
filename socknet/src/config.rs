use crate::{
	endpoint::{self, Endpoint},
	stream,
};
use std::net::SocketAddr;

pub struct Config {
	pub endpoint: endpoint::Config,
	pub address: SocketAddr,
	pub stream_processor: stream::processor::ArcProcessor,
	pub error_sender: stream::error::Sender,
}

impl Config {
	pub fn build(self) -> anyhow::Result<Endpoint> {
		log::info!(target: crate::LOG, "Creating network on {}", self.address);
		match self.endpoint {
			endpoint::Config::Server(config) => {
				let (endpoint, incoming) = quinn::Endpoint::server(config, self.address)?;
				let mut socket = Endpoint::new(endpoint, self.stream_processor, self.error_sender);
				socket.listen_for_connections(incoming);
				Ok(socket)
			}
			endpoint::Config::Client(config) => {
				let mut endpoint = quinn::Endpoint::client(self.address)?;
				endpoint.set_default_client_config(config);
				Ok(Endpoint::new(
					endpoint,
					self.stream_processor,
					self.error_sender,
				))
			}
		}
	}
}
