#[derive(Clone, Copy, Debug)]
pub enum Kind {
	Unidirectional,
	Bidirectional,
	Datagram,
}
impl std::fmt::Display for Kind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Unidirectional => write!(f, "Unidirectional"),
			Self::Bidirectional => write!(f, "Bidirectional"),
			Self::Datagram => write!(f, "Datagram"),
		}
	}
}


