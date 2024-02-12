mod appeal;
mod db;
mod error;
mod types;

use appeal::AppealService;
use rsmev::service::{Message, Service};
use types::{PosEdmsRequest, PosEdmsRequestTypes, PosEdmsResponse, PosEdmsResponseTypes};

use error::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct PosMock {
    service: AppealService,
}

type Files = Vec<std::path::PathBuf>;
impl PosMock {
    pub async fn new(db: &str) -> Self {
        let pg = sqlx::PgPool::connect(db).await.unwrap();
        let repo = db::AppealRepo::new(std::sync::Arc::new(pg));
        Self {
            service: AppealService::new(repo).await,
        }
    }

    async fn handle_appeal_list(
        &self,
        request: types::AppealListRequest,
    ) -> Result<types::AppealListResponse> {
        let client_id = request.client_id;

        let mut appeals = Vec::new();
        if let Some(a) = self.service.next_appeal(client_id).await {
            appeals.push(a);
        }

        let count = appeals.len();
        Ok(types::AppealListResponse {
            status: types::AppealListResponseStatus {
                operation_result: "SUCCESS".to_string(),
                description: None,
            },
            appeals,
            count,
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
