use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Serialize, Clone)]
pub struct PosEdmsResponse {}
