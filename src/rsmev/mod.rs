pub(crate) mod client;
pub(crate) mod extractor;
mod server;

pub type Request = String;
pub type Response = String;

pub use server::serve;
