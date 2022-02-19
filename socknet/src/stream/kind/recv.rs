use crate::utility::PinFutureResultLifetime;

#[doc(hidden)]
mod read;
pub use read::*;

pub mod ongoing;
pub use ongoing::Ongoing;

pub mod datagram;
pub use datagram::Datagram;

pub trait Recv {
	/// Stop accepting data. Discards unread data and notifies the peer to stop transmitting.
	fn stop<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()>;
}
