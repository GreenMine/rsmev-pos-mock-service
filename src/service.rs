use std::fs::File;
use std::future::Future;

#[derive(Debug)]
pub struct Message<C> {
    pub content: C,
    pub files: Vec<File>,
}

// TODO: maybe just add a associated type Response(which may be a result, if it can be failed)
pub trait Service: Send + Sync + 'static {
    type Request: serde::de::DeserializeOwned + Send;
    type Response: serde::Serialize + Clone + Send + Sync;
    type Error: std::error::Error + Clone + Send + Sync;

    fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> impl Future<Output = std::result::Result<Message<Self::Response>, Self::Error>> + Send;
}
