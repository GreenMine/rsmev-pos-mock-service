pub mod body;
pub(crate) mod client;
pub(crate) mod extractor;
mod server;

mod handler_service;

pub use server::serve;

type Result<S> = std::result::Result<body::Body, <S as crate::service::Service>::Error>;
