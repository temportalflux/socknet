use crate::stream::local::AnyBox;

pub enum Datagram {
	Serialized(bytes::Bytes),
	Local(Vec<AnyBox>),
}
impl From<bytes::Bytes> for Datagram {
	fn from(data: bytes::Bytes) -> Self {
		Self::Serialized(data)
	}
}
impl From<Vec<AnyBox>> for Datagram {
	fn from(data: Vec<AnyBox>) -> Self {
		Self::Local(data)
	}
}
