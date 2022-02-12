use super::Active;
use crate::{
	connection::Datagram,
	endpoint::Endpoint,
	stream::{
		kind::{
			recv::{self, ongoing::local::Internal as RecvLocalOngoing},
			send::{self, ongoing::local::Internal as SendLocalOngoing},
		},
		local::{AnyBox, Outgoing},
	},
	utility::PinFutureResultLifetime,
};
use std::{net::SocketAddr, sync::Weak};

pub struct Local {
	pub(crate) endpoint: Weak<Endpoint>,
	pub(crate) uni_streams: Outgoing<RecvLocalOngoing>,
	pub(crate) bi_streams: Outgoing<(SendLocalOngoing, RecvLocalOngoing)>,
	pub(crate) datagrams: Outgoing<Vec<AnyBox>>,
}

impl Active for Local {
	fn remote_address(&self) -> SocketAddr {
		self.endpoint.upgrade().unwrap().address()
	}

	fn peer_identity(&self) -> Option<Box<dyn std::any::Any>> {
		Some(Box::new(vec![self
			.endpoint
			.upgrade()
			.unwrap()
			.certificate()
			.clone()]))
	}

	fn is_local(&self) -> bool {
		true
	}

	fn open_uni<'a>(&'a self) -> PinFutureResultLifetime<'a, send::Ongoing> {
		Box::pin(async move {
			let (send, recv) = async_channel::unbounded::<AnyBox>();
			self.uni_streams.send(Ok(recv)).await?;
			Ok(send.into())
		})
	}

	fn open_bi<'a>(&'a self) -> PinFutureResultLifetime<'a, (send::Ongoing, recv::Ongoing)> {
		Box::pin(async move {
			let (a_send, b_recv) = async_channel::unbounded::<AnyBox>();
			let (b_send, a_recv) = async_channel::unbounded::<AnyBox>();
			self.bi_streams.send(Ok((b_send, b_recv))).await?;
			Ok((
				send::Ongoing::Local(a_send.into()),
				recv::Ongoing::Local(a_recv.into()),
			))
		})
	}

	fn send_datagram(&self, datagram: Datagram) -> anyhow::Result<()> {
		if let Datagram::Local(data) = datagram {
			self.datagrams.try_send(Ok(data))?;
			Ok(())
		} else {
			unimplemented!()
		}
	}

	fn close(&self, _code: u32, _reason: &[u8]) {
		// NO-OP: Local connections mean the application is sending data to itself
		// (Client-On-Top-Of-Server: a situation where the user is hosting a game
		// and the app is both the server for other clients and a client itself),
		// so "closing" this connection has no meaning. You cannot close a connection to yourself.
	}
}
