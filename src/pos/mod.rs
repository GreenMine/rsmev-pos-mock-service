use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::service::{Message, Service};

pub struct Pos {
    i: AtomicUsize,
}

impl Pos {
    pub const fn new() -> Self {
        Self {
            i: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> usize {
        self.i.fetch_add(1, Ordering::Relaxed)
    }
}

// TODO: error was: can't leak private type
#[derive(Debug, serde::Deserialize, Clone)]
pub struct PosRequest {
    #[serde(rename = "@one")]
    one: String,

    #[serde(rename = "@two")]
    two: String,
}

impl Service for Pos {
    type Request = PosRequest;
    type Response = String;
    type Error = Infallible;

    async fn handle(&self, content: Message<Self::Request>) -> crate::service::Result<Self> {
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

        println!("Content: {:?}", content);

        Ok(format!("pos{}", self.next()))
    }
}
