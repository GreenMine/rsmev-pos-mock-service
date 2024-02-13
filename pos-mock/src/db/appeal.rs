use sqlx::{PgPool, Result};
use std::sync::Arc;
use uuid::Uuid;

pub struct AppealRepo {
    pool: Arc<PgPool>,
}

impl AppealRepo {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn get_pending_appeal(&self, client_id: String) -> Result<Appeal> {
        sqlx::query_as!(
            Appeal,
            r#"
                select ap.id, ap.status, ap.content, at.client_id 
                from appeals ap, auth_token at
                where ap.status = 'pending' and ap.auth_id = at.id and at.client_id = $1
                order by ap.created_at asc
                limit 1
            "#r,
            client_id
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn get_appeals_stat(&self) -> Result<Vec<(String, usize)>> {
        use futures_util::stream::StreamExt;
        let mut s = sqlx::query!(
            r#"
            select at.client_id, stat.amount from (
	            select ap.auth_id, count(*) as amount
	            from appeals ap
	            group by ap.auth_id
            ) stat, auth_token at
            where stat.auth_id = at.id
            order by amount desc
            "#r,
        )
        .fetch(&*self.pool);

        let mut records = Vec::new();

        while let Some(Ok(record)) = s.next().await {
            records.push((record.client_id.unwrap(), record.amount.unwrap() as usize));
        }

        Ok(records)
    }
}

pub struct Appeal {
    pub id: i32,
    pub status: AppealStatus,
    pub content: Option<sqlx::types::JsonValue>,
    pub client_id: Option<String>,
}

pub enum AppealStatus {
    Pending,
    Confirming,
    Accepted,
}

impl From<AppealStatus> for String {
    fn from(value: AppealStatus) -> Self {
        match value {
            AppealStatus::Pending => "pending",
            AppealStatus::Confirming => "confirming",
            AppealStatus::Accepted => "accepted",
        }
        .to_string()
    }
}

impl From<String> for AppealStatus {
    fn from(value: String) -> Self {
        match &value[..] {
            "pending" => AppealStatus::Pending,
            "confirming" => AppealStatus::Confirming,
            "accepted" => AppealStatus::Accepted,
            _ => unreachable!(),
        }
    }
}
