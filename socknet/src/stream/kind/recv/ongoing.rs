use crate::stream::kind::Locality;

#[doc(hidden)]
mod remote;
pub use remote::Remote;

#[doc(hidden)]
pub(crate) mod local;
pub use local::Local;

/// An incoming stream that can continue to read data as long as the connection is available.
/// Once opened, [`read`](Read) methods can be used to receive data.
pub type Ongoing = Locality<Remote, Local>;

impl From<quinn::RecvStream> for Ongoing {
	fn from(stream: quinn::RecvStream) -> Self {
		Self::Remote(stream.into())
	}
}

impl From<local::Internal> for Ongoing {
	fn from(stream: local::Internal) -> Self {
		Self::Local(stream.into())
	}
}
