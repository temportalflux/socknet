use crate::stream::{self, kind::Locality};

#[doc(hidden)]
mod remote;
pub use remote::*;

#[doc(hidden)]
mod local;
pub use local::*;

/// An incoming buffer of bytes which was sent to this connection when
/// [`finish`](crate::stream::kind::Send::finish) was called.
/// Use [`read`](super::Read) methods can be used to receive data, which is read from a fixed array of bytes.
pub type Datagram = Locality<Remote, Local>;

impl From<bytes::Bytes> for Datagram {
	fn from(stream: bytes::Bytes) -> Self {
		Self::Remote(stream.into())
	}
}

impl From<Vec<stream::local::AnyBox>> for Datagram {
	fn from(stream: Vec<stream::local::AnyBox>) -> Self {
		Self::Local(stream.into())
	}
}
