use crate::stream::kind::Locality;

#[doc(hidden)]
mod remote;
pub use remote::*;

#[doc(hidden)]
mod local;
pub use local::*;

/// An outgoing buffer of bytes which is sent to its connection when [`finish`](crate::stream::kind::Send::finish) is called.
///
/// IMPORTANT: This stream does not operate like a [`Send stream`](crate::stream::kind::Send).
/// You MUST call [`finish`](crate::stream::kind::Send::finish) to transmit the datagram (unreliably) to the peer.
/// See [`quinn::Connection::send_datagram`](quinn::Connection::send_datagram) for more information.
pub type Datagram = Locality<Remote, Local>;
