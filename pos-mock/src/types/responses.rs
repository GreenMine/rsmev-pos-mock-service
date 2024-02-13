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
pub struct Appeal {
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
    pub attachments: Vec<File>,
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

impl TryFrom<crate::db::Appeal> for Appeal {
    type Error = ();

    fn try_from(value: crate::db::Appeal) -> Result<Self, Self::Error> {
        fn new_object_value_case(value: serde_json::Value) -> serde_json::Value {
            fn change_first_symbol_case(str: &mut String) {
                // SAFETY: only ascii symbols can be provided in Object
                let bytes = unsafe { str.as_bytes_mut() };

                assert!(bytes[0].is_ascii());
                bytes[0] = bytes[0].to_ascii_uppercase();
            }

            if let serde_json::Value::Object(obj) = value {
                let mut map = serde_json::Map::with_capacity(obj.len());
                for (mut key, value) in obj.into_iter() {
                    change_first_symbol_case(&mut key);
                    map.insert(key, new_object_value_case(value));
                }

                return serde_json::Value::Object(map);
            }
            return value;
        }

        let content = value.content.unwrap();
        let content = new_object_value_case(content);

        serde_json::from_value::<Appeal>(content).map_err(|_| ())
    }
}
