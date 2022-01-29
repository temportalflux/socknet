//! TODO DELETE ME

use crate::stream;

struct TestUniBuilder {}
impl stream::Identifier for TestUniBuilder {
	fn unique_id() -> &'static str {
		"uni"
	}
}
impl stream::send::Builder for TestUniBuilder {
	type Opener = stream::uni::Opener;
}
impl stream::recv::Builder for TestUniBuilder {
	type Extractor = stream::uni::Extractor;
	type Receiver = TestUniRecv;
}

struct TestUniSend {}
impl stream::handler::Initiator for TestUniSend {
	type Builder = TestUniBuilder;
}
impl From<stream::send::Context<TestUniBuilder>> for TestUniSend {
	fn from(_context: stream::send::Context<TestUniBuilder>) -> Self {
		Self {}
	}
}
struct TestUniRecv;
impl From<stream::recv::Context<TestUniBuilder>> for TestUniRecv {
	fn from(_context: stream::recv::Context<TestUniBuilder>) -> Self {
		Self {}
	}
}
impl stream::handler::Receiver for TestUniRecv {
	type Builder = TestUniBuilder;
	fn receive(self) {}
}

struct TestBiBuilder {}
impl stream::Identifier for TestBiBuilder {
	fn unique_id() -> &'static str {
		"bi"
	}
}
impl stream::send::Builder for TestBiBuilder {
	type Opener = stream::bi::Opener;
}
impl stream::recv::Builder for TestBiBuilder {
	type Extractor = stream::bi::Extractor;
	type Receiver = TestBiChannel;
}

struct TestBiChannel {}
impl stream::handler::Initiator for TestBiChannel {
	type Builder = TestBiBuilder;
}
impl From<stream::Context<TestBiBuilder, stream::kind::Bidirectional>> for TestBiChannel {
	fn from(_context: stream::Context<TestBiBuilder, stream::kind::Bidirectional>) -> Self {
		Self {}
	}
}
impl stream::handler::Receiver for TestBiChannel {
	type Builder = TestBiBuilder;
	fn receive(self) {}
}

struct TestDatagramBuilder {}
impl stream::Identifier for TestDatagramBuilder {
	fn unique_id() -> &'static str {
		"datagram"
	}
}
impl stream::recv::Builder for TestDatagramBuilder {
	type Extractor = stream::datagram::Extractor;
	type Receiver = TestDatagramRecv;
}

struct TestDatagramRecv {}
impl From<stream::recv::Context<TestDatagramBuilder>> for TestDatagramRecv {
	fn from(_context: stream::recv::Context<TestDatagramBuilder>) -> Self {
		Self {}
	}
}
impl stream::handler::Receiver for TestDatagramRecv {
	type Builder = TestDatagramBuilder;
	fn receive(self) {}
}
