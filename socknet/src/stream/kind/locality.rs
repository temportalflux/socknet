use crate::{
	stream::kind::{
		recv::{Read, Recv},
		send::{Send, Write},
	},
	utility::PinFutureResultLifetime,
};

pub enum Locality<R, L> {
	Remote(R),
	Local(L),
}

impl<R, L> Send for Locality<R, L>
where
	R: Send + std::marker::Send + 'static,
	L: Send + std::marker::Send + 'static,
{
	fn finish<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		match self {
			Self::Remote(remote) => remote.finish(),
			Self::Local(local) => local.finish(),
		}
	}
}

impl<R, L> Recv for Locality<R, L>
where
	R: Recv + std::marker::Send + 'static,
	L: Recv + std::marker::Send + 'static,
{
	fn stop<'a>(&'a mut self) -> PinFutureResultLifetime<'a, ()> {
		match self {
			Self::Remote(remote) => remote.stop(),
			Self::Local(local) => local.stop(),
		}
	}
}

impl<R, L> Write for Locality<R, L>
where
	R: Write + std::marker::Send + 'static,
	L: Write + std::marker::Send + 'static,
{
	fn write_exact<'a>(&'a mut self, buf: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		match self {
			Self::Remote(remote) => remote.write_exact(buf),
			Self::Local(local) => local.write_exact(buf),
		}
	}

	fn write_size<'a>(&'a mut self, len: usize) -> PinFutureResultLifetime<'a, ()> {
		match self {
			Self::Remote(remote) => remote.write_size(len),
			Self::Local(local) => local.write_size(len),
		}
	}

	fn write_bytes<'a>(&'a mut self, data: &'a [u8]) -> PinFutureResultLifetime<'a, ()> {
		match self {
			Self::Remote(remote) => remote.write_bytes(data),
			Self::Local(local) => local.write_bytes(data),
		}
	}

	fn write<'a, T>(&'a mut self, data: &'a T) -> PinFutureResultLifetime<'a, ()>
	where
		Self: std::marker::Send,
		T: 'static + serde::Serialize + Clone + std::marker::Send + Sync,
	{
		match self {
			Self::Remote(remote) => remote.write(data),
			Self::Local(local) => local.write(data),
		}
	}
}

impl<R, L> Read for Locality<R, L>
where
	R: Read + std::marker::Send + 'static,
	L: Read + std::marker::Send + 'static,
{
	fn read_exact<'a>(&'a mut self, byte_count: usize) -> PinFutureResultLifetime<'a, Vec<u8>> {
		match self {
			Self::Remote(remote) => remote.read_exact(byte_count),
			Self::Local(local) => local.read_exact(byte_count),
		}
	}

	fn read_size<'a>(&'a mut self) -> PinFutureResultLifetime<'a, usize> {
		match self {
			Self::Remote(remote) => remote.read_size(),
			Self::Local(local) => local.read_size(),
		}
	}

	fn read_bytes<'a>(&'a mut self) -> PinFutureResultLifetime<'a, Vec<u8>> {
		match self {
			Self::Remote(remote) => remote.read_bytes(),
			Self::Local(local) => local.read_bytes(),
		}
	}

	fn read<'a, T>(&'a mut self) -> PinFutureResultLifetime<'a, T>
	where
		T: serde::de::DeserializeOwned + Sized + std::marker::Send + Sync + 'static,
	{
		match self {
			Self::Remote(remote) => remote.read(),
			Self::Local(local) => local.read(),
		}
	}
}
