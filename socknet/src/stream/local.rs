pub type AnyBox = Box<dyn std::any::Any + std::marker::Send + Sync + 'static>;
#[allow(dead_code)]
pub type Incoming<T> = async_channel::Receiver<Result<T, quinn::ConnectionError>>;
pub type Outgoing<T> = async_channel::Sender<Result<T, quinn::ConnectionError>>;
