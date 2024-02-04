pub(crate) mod client;
pub(crate) mod extractor;
mod message;
mod server;

pub type Request = String;

pub use message::Message;
pub use server::serve;
