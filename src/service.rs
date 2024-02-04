use std::fs::File;

use crate::rsmev::Body as RsmevBody;

pub type Result<S> = std::result::Result<<S as Service>::Response, <S as Service>::Error>;

#[derive(Debug)]
pub struct Message<C> {
    content: C,
    files: Vec<File>,
}

impl<'de, C: serde::Deserialize<'de>> Message<C> {
    pub fn from_rsmev_body(body: RsmevBody) -> Self {
        Self {
            content: body.xml.deserialize().unwrap(),
            files: Vec::new(),
        }
    }
}

// TODO: maybe just add a associated type Response(which may be a result, if it can be failed)
pub trait Service: Send + Sync + 'static {
    type Request: serde::de::DeserializeOwned + Send;
    type Response: serde::Serialize + Clone + Send + Sync;
    type Error: std::error::Error + Clone + Send + Sync;

    fn process(
        &self,
        message: RsmevBody,
    ) -> impl std::future::Future<Output = Result<Self>> + Send {
        let content = Self::parse(message);

        self.handle(content)
    }

    fn parse(message: RsmevBody) -> Message<Self::Request> {
        Message::from_rsmev_body(message)
    }
    fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> impl std::future::Future<Output = Result<Self>> + Send;
}
