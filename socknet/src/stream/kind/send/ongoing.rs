use crate::stream::kind::Locality;

#[doc(hidden)]
mod remote;
pub use remote::Remote;

#[doc(hidden)]
pub(crate) mod local;
pub use local::Local;

/// An outgoing stream that can continue to send data as long as the connection is available.
/// Once opened, [`write`](Write) methods can be used to transmit data.
pub type Ongoing = Locality<Remote, Local>;

impl From<quinn::SendStream> for Ongoing {
	fn from(stream: quinn::SendStream) -> Self {
		Self::Remote(stream.into())
	}
}

impl From<local::Internal> for Ongoing {
	fn from(stream: local::Internal) -> Self {
		Self::Local(stream.into())
	}
}
