use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PosEdmsRequest {
    #[serde(rename = "$value")]
    pub request: PosEdmsRequestTypes,
}

#[derive(Debug, Deserialize)]
pub enum PosEdmsRequestTypes {
    AppealListRequest(#[serde(rename = "$value")] AppealListRequest),
}

#[derive(Debug, Deserialize)]
pub struct AppealListRequest {
    #[serde(rename = "ClientId")]
    pub client_id: Uuid,
}
