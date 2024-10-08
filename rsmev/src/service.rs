// TODO: maybe move it to rsmev_service library

// TODO: Using tower library is ideomatic variant for service
// @see https://docs.rs/tower/latest/tower/

use std::future::Future;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Message<C> {
    pub content: C,
    pub files: Vec<PathBuf>,
}

// TODO: maybe just add a associated type Response(which may be a result, if it can be failed)
pub trait Service: Send + Sync + 'static {
    type Request: serde::de::DeserializeOwned + Send;
    type Response: serde::Serialize + Send + Sync;
    type Error: std::error::Error + Send + Sync;

    fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> impl Future<Output = std::result::Result<Message<Self::Response>, Self::Error>> + Send;
}
