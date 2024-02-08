pub mod body;
pub(crate) mod client;
pub(crate) mod extractor;
mod serve;

mod handler_service;

pub use serve::serve;

type Result<S> = std::result::Result<body::Body, <S as crate::service::Service>::Error>;
