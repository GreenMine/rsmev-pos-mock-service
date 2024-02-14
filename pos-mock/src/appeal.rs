use std::str::FromStr;

use crate::db::appeal::AppealStatus;
use crate::db::AppealRepo;
use crate::types::Appeal;
use dashmap::DashMap;
use tokio::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

pub struct AppealService {
    repo: AppealRepo,
    popularity_map: DashMap<uuid::Uuid, (Instant, Duration)>,
}

const BASE_APPEAL_DIFF_TIME: usize = 0;

impl AppealService {
    pub async fn new(repo: AppealRepo) -> Self {
        let popularity_map = Self::calculate_popularity_map(&repo).await;

        Self {
            repo,
            popularity_map,
        }
    }

    async fn calculate_popularity_map(
        repo: &AppealRepo,
    ) -> DashMap<uuid::Uuid, (Instant, Duration)> {
        let stat = repo.get_appeals_stat().await.unwrap();
        let max = stat.iter().map(|(_, a)| *a).max().unwrap();

        let now = Instant::now();
        stat.into_iter()
            .map(|(uuid, amount)| {
                let uuid = uuid::Uuid::from_str(&uuid).expect("valid uuid");
                let coef = max as f32 / amount as f32;

                let message_duration = BASE_APPEAL_DIFF_TIME as f32 * coef;
                let message_duration = Duration::from_millis(message_duration as u64);

                (uuid, (now, message_duration))
            })
            .collect::<DashMap<_, _>>()
    }

    pub async fn next_appeal(&self, client_id: Uuid) -> Option<Appeal> {
        if self.has_appeal(client_id) {
            match self.get_appeal(client_id).await {
                Ok(a) => Some(a),
                Err(e) => {
                    tracing::error!(error = ?e);
                    println!("Error: {e:?}");
                    None
                }
            }
        } else {
            None
        }
    }

    async fn get_appeal(&self, client_id: Uuid) -> Result<Appeal, crate::Error> {
        let appeal = self.repo.get_pending_appeal(client_id.to_string()).await?;
        let appeal_id = appeal.id;

        let converted = Appeal::try_from(appeal)?;
        self.repo
            .update_status(appeal_id, AppealStatus::Confirming)
            .await?;

        Ok(converted)
    }

    pub fn has_appeal(&self, client_id: Uuid) -> bool {
        use dashmap::mapref::entry::Entry;
        match self.popularity_map.entry(client_id) {
            Entry::Occupied(mut entry) => {
                let (now, duration) = entry.get_mut();

                if now.elapsed() > *duration {
                    *now = Instant::now();
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(_) => false,
        }
    }
}
