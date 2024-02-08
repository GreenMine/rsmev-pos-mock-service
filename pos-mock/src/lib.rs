mod types;

use std::convert::Infallible;

use rsmev::service::{Message, Service};
use types::{PosEdmsRequest, PosEdmsResponse};

pub struct PosMock {}

impl PosMock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Service for PosMock {
    type Request = PosEdmsRequest;
    type Response = PosEdmsResponse;
    type Error = Infallible;

    async fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> Result<Message<Self::Response>, Self::Error> {
        println!("Content: {:?}", content);

        Ok(Message {
            content: PosEdmsResponse {},
            files: Vec::new(),
        })
    }
}
