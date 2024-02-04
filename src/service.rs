use std::fs::File;
use std::future::Future;

use crate::rsmev::body::{Body as RsmevBody, EncodedXml};

pub type Result<S> = std::result::Result<RsmevBody, <S as Service>::Error>;

#[derive(Debug)]
pub struct Message<C> {
    pub content: C,
    pub files: Vec<File>,
}

impl<'de, C: serde::Deserialize<'de>> Message<C> {
    pub fn from_rsmev_body(body: RsmevBody) -> Self {
        Self {
            content: body.xml.deserialize().unwrap(),
            files: Vec::new(),
        }
    }
}

impl<C: serde::Serialize> Message<C> {
    pub fn to_rsmev_body(self) -> RsmevBody {
        RsmevBody {
            xml: EncodedXml::serialize(&self.content).unwrap(),
            files: Vec::new(),
        }
    }
}

// TODO: maybe just add a associated type Response(which may be a result, if it can be failed)
pub trait Service: Send + Sync + 'static {
    type Request: serde::de::DeserializeOwned + Send;
    type Response: serde::Serialize + Clone + Send + Sync;
    type Error: std::error::Error + Clone + Send + Sync;

    fn process(&self, message: RsmevBody) -> impl Future<Output = Result<Self>> + Send {
        async {
            let content = Self::parse(message);
            let response = self.handle(content).await;

            response.map(Message::to_rsmev_body)
        }
    }

    fn parse(message: RsmevBody) -> Message<Self::Request> {
        Message::from_rsmev_body(message)
    }
    fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> impl Future<Output = std::result::Result<Message<Self::Response>, Self::Error>> + Send;
}
