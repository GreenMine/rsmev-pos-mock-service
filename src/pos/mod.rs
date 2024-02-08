use std::{
    convert::Infallible,
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::service::{Message, Service};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Pos {}

impl Pos {
    pub const fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PosEdmsRequest {
    #[serde(rename = "$value")]
    request: PosEdmsRequestTypes,
}

#[derive(Debug, Deserialize, Clone)]
pub enum PosEdmsRequestTypes {
    AppealListRequest(#[serde(rename = "$value")] AppealListRequest),
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppealListRequest {
    #[serde(rename = "ClientId")]
    client_id: Uuid,
}

#[derive(Serialize, Clone)]
pub struct PosResponse {
    #[serde(rename = "$text")]
    response: String,

    #[serde(rename = "@status")]
    status: i32,
}

impl Service for Pos {
    type Request = PosEdmsRequest;
    type Response = PosResponse;
    type Error = Infallible;

    async fn handle(
        &self,
        content: Message<Self::Request>,
    ) -> Result<Message<Self::Response>, Self::Error> {
        println!("Content: {:?}", content);

        Ok(Message {
            content: PosResponse {
                response: "ALL FINE".to_string(),
                status: 200,
            },
            files: Vec::new(),
        })
    }
}
