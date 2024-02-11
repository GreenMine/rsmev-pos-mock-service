mod appeal;
mod db;
mod error;
mod types;

use appeal::AppealService;
use rsmev::service::{Message, Service};
use types::{PosEdmsRequest, PosEdmsRequestTypes, PosEdmsResponse, PosEdmsResponseTypes};

use error::Error;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Error>;

pub struct PosMock {}

type Files = Vec<std::path::PathBuf>;
impl PosMock {
    pub const fn new() -> Self {
        Self {}
    }

    async fn handle_appeal_list(
        &self,
        request: types::AppealListRequest,
    ) -> Result<types::AppealListResponse> {
        let appeal = types::Appeal {
            id: 563066016,
            description: "Территория современной Липецкой области расположена в лесостепной зоне на границе лесов и степей и на протяжении тысячелетий со сменой периодов похолоданий и потеплений леса и степи много раз передвигалась по этим землям с севера на юг и обратно. Соответственно этому и волны миграций древнейших лесных и степных народов много раз сменяли здесь друг друга. По данным археологов территория на которой в данное время располагается Липецкая область обживалась людьми ещё со времён верхнего палеолита. Наиболее известными обнаруженными археологами стоянками древних людей являются: стоянка Гагарино относящаяся к граветтской культуре[12] и стоянка «Замятино 14» в Задонском районе относящаяся к эпиграветтской культуре[13]. ".to_string(),
            subject_id: 114,
            subject_name: "Автомобильные дороги".to_string(),
            subsubject_id: 2654,
            subsubject_name: "Дорожная разметка".to_string(),
            fact_name: String::new(),
            answer_at: chrono::offset::Local::now(),
            fast_track: false,
            created_at: chrono::offset::Local::now(),
            region_id: uuid::uuid!("1490490e-49c5-421c-9572-5673ba5d80c8"),
            region_name: "Липецкая область".to_string(),
            address: String::new(),
            opa_id: 725595,
            opa_name: "УПРАВЛЕНИЕ ДЕЛАМИ АДМИНИСТРАЦИИ ЛИПЕЦКОЙ ОБЛАСТИ".to_string(),
            shared: false,
            applicant: types::AppealApplicant {
                surname: "Иванов".to_string(),
                name: "Иван".to_string(),
                patronymic: "Иванович".to_string(),
                email: "bnskot@mail.ru".to_string(),
                phone: "+7 (121) 212-12-12".to_string(),
                post_address: String::new(),
                send_with_russia_post: false,
                post_address_flat: String::new()
            },
            attachments: vec![types::File {
                file_id: Uuid::new_v4()
            }],
            coordinates: "52.60882000,39.59922000".to_string(),
            confidential: true,
            work_log: Uuid::new_v4()
        };

        Ok(types::AppealListResponse {
            status: types::AppealListResponseStatus {
                operation_result: "SUCCESS".to_string(),
                description: None,
            },
            appeals: vec![appeal],
            count: 1,
        })
    }
}

impl Service for PosMock {
    type Request = PosEdmsRequest;
    type Response = PosEdmsResponse;
    type Error = Error;

    async fn handle(&self, content: Message<Self::Request>) -> Result<Message<Self::Response>> {
        println!("Content: {:?}", content);

        let Message { content, files } = content;
        let (response, files) = match content.request {
            PosEdmsRequestTypes::AppealListRequest(rq) => (
                PosEdmsResponseTypes::AppealListResponse(self.handle_appeal_list(rq).await?),
                None,
            ),
        };

        Ok(Message {
            content: PosEdmsResponse { response },
            files: files.unwrap_or_default(),
        })
    }
}
