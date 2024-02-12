use std::str::FromStr;

use crate::db::{appeal::Appeal as DbAppeal, AppealRepo};
use crate::types::Appeal;
use dashmap::DashMap;
use tokio::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

pub struct AppealService {
    repo: AppealRepo,
    popularity_map: DashMap<uuid::Uuid, (Instant, Duration)>,
}

const BASE_APPEAL_DIFF_TIME: usize = 10000;

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
            self.repo
                .get_pending_appeal(client_id.to_string())
                .await
                .ok()
                .and_then(|v| Self::convert_db_appeal(v).ok())
        } else {
            None
        }
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

    fn convert_db_appeal(appeal: DbAppeal) -> Result<Appeal, ()> {
        Ok(Appeal {
            id: appeal["id"],
            description: todo!(),
            subject_id: todo!(),
            subject_name: todo!(),
            subsubject_id: todo!(),
            subsubject_name: todo!(),
            fact_name: todo!(),
            answer_at: todo!(),
            fast_track: todo!(),
            created_at: todo!(),
            region_id: todo!(),
            region_name: todo!(),
            address: todo!(),
            opa_id: todo!(),
            opa_name: todo!(),
            shared: todo!(),
            applicant: todo!(),
            attachments: todo!(),
            coordinates: todo!(),
            confidential: todo!(),
            work_log: todo!(),
        });
        unimplemented!()
    }
}
