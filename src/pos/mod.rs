use std::{
    convert::Infallible,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::service::{Message, Service};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
#[derive(Debug, Deserialize, Clone)]
pub struct PosRequest {
    #[serde(rename = "@one")]
    one: String,

    #[serde(rename = "@two")]
    two: String,
}

#[derive(Serialize, Clone)]
pub struct PosResponse {
    #[serde(rename = "$text")]
    response: String,

    #[serde(rename = "@status")]
    status: i32,
}

impl Service for Pos {
    type Request = PosRequest;
    type Response = PosResponse;
    type Error = Infallible;

    async fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> Result<Message<Self::Response>, Self::Error> {
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

        println!("Content: {:?}", content);

        let mut file_path = PathBuf::new();
        file_path.push("/tmp");
        file_path.push(format!("{}.txt", Uuid::new_v4()));

        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write(b"hello, from pos!").unwrap();
        }

        Ok(Message {
            content: PosResponse {
                response: "ALL FINE".to_string(),
                status: 200,
            },
            files: vec![file_path],
        })
    }
}
