#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Cannot get Connection, it has been dropped already.")]
	ConnectionDropped,
	#[error("Connection has no identity.")]
	NoIdentity,
	#[error("Connection's identity is not a list of certificates.")]
	IdentityIsNotCertificate,
	#[error("Connection's identity certificate list is empty.")]
	CertificateIdentityIsEmpty,
}
