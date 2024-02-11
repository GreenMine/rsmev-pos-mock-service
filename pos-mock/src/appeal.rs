use std::str::FromStr;

use crate::db::{appeal::Appeal, AppealRepo};
use dashmap::DashMap;
use tokio::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

pub struct AppealService {
    repo: AppealRepo,
    popularity_map: DashMap<uuid::Uuid, (Instant, f32)>,
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

    async fn calculate_popularity_map(repo: &AppealRepo) -> DashMap<uuid::Uuid, (Instant, f32)> {
        let stat = repo.get_appeals_stat().await.unwrap();
        let max = stat.iter().map(|(_, a)| *a).max().unwrap();

        stat.into_iter()
            .map(|(uuid, amount)| {
                let uuid = uuid::Uuid::from_str(&uuid).expect("valid uuid");
                let coef = max as f32 / amount as f32;

                (uuid, (Instant::now(), coef))
            })
            .collect::<DashMap<_, _>>()
    }

    pub async fn next_appeal(&self, client_id: Uuid) -> Option<Appeal> {
        if self.has_appeal(client_id) {
            self.repo
                .get_pending_appeal(client_id.to_string())
                .await
                .ok()
        } else {
            None
        }
    }

    pub fn has_appeal(&self, client_id: Uuid) -> bool {
        use dashmap::mapref::entry::Entry;
        match self.popularity_map.entry(client_id) {
            Entry::Occupied(mut entry) => {
                let (now, coef) = entry.get_mut();

                let must_diff = BASE_APPEAL_DIFF_TIME as f32 * (*coef);
                let must_diff = Duration::from_millis(must_diff as u64);
                if now.elapsed() > must_diff {
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
