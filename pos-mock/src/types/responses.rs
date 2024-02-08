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
    pub(crate) status: AppealListResponseStatus,
    pub(crate) appeals: Vec<Appeal>,
    pub(crate) count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppealListResponseStatus {
    pub(crate) operation_result: String,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Appeal {
    pub(crate) id: u64,
    pub(crate) description: String,
    pub(crate) subject_id: u64,
    pub(crate) subject_name: String,
    pub(crate) subsubject_id: u64,
    pub(crate) subsubject_name: String,
    pub(crate) fact_name: String,
    pub(crate) answer_at: DateTime,
    pub(crate) fast_track: bool,
    pub(crate) created_at: DateTime,
    pub(crate) region_id: Uuid,
    pub(crate) region_name: String,
    pub(crate) address: String,
    pub(crate) opa_id: u64,
    pub(crate) opa_name: String,
    pub(crate) shared: bool,
    pub(crate) applicant: AppealApplicant,
    pub(crate) attachments: Vec<File>,
    pub(crate) coordinates: String,
    pub(crate) confidential: bool,
    pub(crate) work_log: Uuid,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    pub(crate) file_id: Uuid,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppealApplicant {
    pub(crate) surname: String,
    pub(crate) name: String,
    pub(crate) patronymic: String,
    pub(crate) email: String,
    pub(crate) phone: String,
    pub(crate) post_address: String,
    pub(crate) send_with_russia_post: bool,
    pub(crate) post_address_flat: String,
}
