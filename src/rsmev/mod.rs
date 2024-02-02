pub(crate) mod client;
pub(crate) mod extractor;
mod server;

pub type Request = String;

pub use server::serve;
