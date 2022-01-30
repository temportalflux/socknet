use crate::utility::PinFutureResultLifetime;

#[doc(hidden)]
mod write;
pub use write::*;

pub mod ongoing;
pub use ongoing::Ongoing;

pub mod datagram;
pub use datagram::Datagram;

pub trait Send {
	/// Finishes the stream.
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()>;
}
