use crate::LOG;
use std::sync::{Arc, Mutex};

pub use tokio::task::JoinHandle;

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

	pub fn spawn<T>(&self, future: T)
	where
		T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
	{
		self.push(crate::utility::spawn(future));
	}

	pub fn push(&self, handle: JoinHandle<()>) {
		self.0.lock().unwrap().push(handle);
	}
}

pub mod bytes {
	pub use rmp_serde::{from_read_ref, to_vec};
}

pub fn spawn<T>(future: T) -> JoinHandle<()>
where
	T: futures::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
	tokio::task::spawn(async move {
		if let Err(err) = future.await {
			log::error!(target: LOG, "{}", err);
		}
	})
}

pub fn fingerprint(certificate: &rustls::Certificate) -> String {
	use base64ct::{Base64UrlUnpadded, Encoding};
	use sha2::{Digest, Sha256};

	let mut hasher = Sha256::new();
	hasher.update(&certificate.0[..]);
	let hash = hasher.finalize();

	Base64UrlUnpadded::encode_string(&hash)
}
