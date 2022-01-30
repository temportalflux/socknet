use super::Active;
use crate::{
	connection::Datagram,
	stream::kind::{recv, send},
	utility::PinFutureResultLifetime,
};
use std::net::SocketAddr;

pub struct Remote(pub(crate) quinn::Connection);

impl Active for Remote {
	fn remote_address(&self) -> SocketAddr {
		self.0.remote_address()
	}

	fn peer_identity(&self) -> Option<Box<dyn std::any::Any>> {
		self.0.peer_identity()
	}

	fn is_local(&self) -> bool {
		false
	}

	fn open_uni<'a>(&'a self) -> PinFutureResultLifetime<'a, send::Ongoing> {
		Box::pin(async move {
			let send = self.0.open_uni().await?;
			Ok(send.into())
		})
	}

	fn open_bi<'a>(&'a self) -> PinFutureResultLifetime<'a, (send::Ongoing, recv::Ongoing)> {
		Box::pin(async move {
			let (send, recv) = self.0.open_bi().await?;
			Ok((
				send::Ongoing::Remote(send.into()),
				recv::Ongoing::Remote(recv.into()),
			))
		})
	}

	fn send_datagram(&self, datagram: Datagram) -> anyhow::Result<()> {
		if let Datagram::Serialized(bytes) = datagram {
			self.0.send_datagram(bytes)?;
			Ok(())
		} else {
			unimplemented!()
		}
	}

	fn close(&self, code: u32, reason: &[u8]) {
		self.0.close(code.into(), reason);
	}
}
