use std::fs::File;

use crate::rsmev::Message as RsmevMessage;

pub type Result<S> = std::result::Result<<S as Service>::Response, <S as Service>::Error>;

#[derive(Debug)]
pub struct Message<C> {
    content: C,
    files: Vec<File>,
}

impl<'de, C: serde::Deserialize<'de>> Message<C> {
    pub fn from_rsmev(message: RsmevMessage) -> Self {
        Self {
            content: message.xml.deserialize().unwrap(),
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
        message: RsmevMessage,
    ) -> impl std::future::Future<Output = Result<Self>> + Send {
        let content = Self::parse(message);

        self.handle(content)
    }

    fn parse(message: RsmevMessage) -> Message<Self::Request> {
        Message::from_rsmev(message)
    }
    fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> impl std::future::Future<Output = Result<Self>> + Send;
}
