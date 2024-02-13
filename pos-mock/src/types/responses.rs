use serde::Serialize;
use uuid::Uuid;

type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Serialize, Clone)]
pub struct PosEdmsResponse {
    #[serde(rename = "$value")]
    pub response: PosEdmsResponseTypes,
}

#[derive(Debug, Serialize, Clone)]
pub enum PosEdmsResponseTypes {
    AppealListResponse(#[serde(rename = "$value")] AppealListResponse),
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppealListResponse {
    pub status: AppealListResponseStatus,
    pub appeals: Vec<Appeal>,
    pub count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppealListResponseStatus {
    pub operation_result: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Appeal<Attachment = File> {
    pub id: u64,
    pub description: String,
    pub subject_id: u64,
    pub subject_name: String,
    pub subsubject_id: u64,
    pub subsubject_name: String,
    pub fact_name: Option<String>,
    pub answer_at: DateTime,
    pub fast_track: bool,
    pub created_at: DateTime,
    pub region_id: Uuid,
    pub region_name: String,
    pub address: String,
    pub opa_id: u64,
    pub opa_name: String,
    pub shared: bool,
    pub applicant: AppealApplicant,
    pub attachments: Vec<Attachment>,
    pub coordinates: String,
    #[serde(default)]
    pub confidential: bool,
    pub work_log: Option<Uuid>,
}

#[derive(Debug, Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    pub file_id: Uuid,
}

#[derive(Debug, Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppealApplicant {
    pub surname: String,
    pub name: String,
    pub patronymic: String,
    pub email: String,
    pub phone: String,
    pub post_address: String,
    pub send_with_russia_post: bool,
    #[serde(default)]
    pub post_address_flat: String,
}
