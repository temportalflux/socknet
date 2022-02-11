use std::{
	pin::Pin,
	sync::{Arc, Mutex},
};

pub use tokio::task::JoinHandle;

/// An annonymous, pinned, future which outputs a result with some generic success type and the [`anyhow error type`](anyhow::Error).
pub type PinFutureResult<T> = PinFutureResultLifetime<'static, T>;
pub type PinFutureResultLifetime<'l, T> =
	Pin<Box<dyn futures_util::future::Future<Output = anyhow::Result<T>> + 'l + Send>>;

pub struct JoinHandleList(Arc<Mutex<Vec<JoinHandle<()>>>>);

impl Drop for JoinHandleList {
	fn drop(&mut self) {
		let mut handles = self.0.lock().unwrap();
		for handle in handles.drain(..) {
			handle.abort();
		}
	}
}

impl JoinHandleList {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(Vec::new())))
	}

	pub fn with_capacity(count: usize) -> Self {
		Self(Arc::new(Mutex::new(Vec::with_capacity(count))))
	}

	pub fn spawn<T>(&self, target: String, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		self.push(crate::utility::spawn(target, future));
	}

	pub fn push(&self, handle: JoinHandle<()>) {
		self.0.lock().unwrap().push(handle);
	}
}

pub fn spawn<T>(target: String, future: T) -> JoinHandle<()>
where
	T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
	tokio::task::spawn(async move {
		if let Err(err) = future.await {
			log::error!(target: &target, "Error: {:?}", err);
		}
	})
}

pub fn fingerprint(certificate: &rustls::Certificate) -> String {
	use sha2::{Digest, Sha256};

	let mut hasher = Sha256::new();
	hasher.update(&certificate.0[..]);
	let hash = hasher.finalize();

	encode_string(&hash)
}

pub fn encode_string(bytes: &[u8]) -> String {
	use base64ct::{Base64UrlUnpadded, Encoding};
	Base64UrlUnpadded::encode_string(&bytes)
}

pub fn decode_bytes(encoded: &str) -> anyhow::Result<Vec<u8>> {
	use base64ct::{Base64UrlUnpadded, Encoding};
	Ok(Base64UrlUnpadded::decode_vec(&encoded)?)
}
