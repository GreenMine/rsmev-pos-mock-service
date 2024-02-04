use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
pub use tokio::net::TcpListener;
use uuid::Uuid;

use dashmap::DashMap;

use super::{client::Client, extractor::HeaderNodeId, Message};
use crate::service::Service;

pub async fn serve<S: Service>(listener: TcpListener, service: S) -> Result<(), std::io::Error> {
    let state = Arc::new(Rsmev::new(service));

    let rsmev_routes = Router::new()
        .route("/send", get(send_request))
        .route("/get", get(get_request))
        .route("/confirm/:request_id", get(confirm_request))
        .with_state(state);

    let routes = Router::new().nest("/api/smev/:entrypoint_id", rsmev_routes);
    axum::serve(listener, routes).await
}

#[derive(serde::Deserialize, Debug)]
struct SendRequest {
    #[serde(flatten)]
    message: Message,
}

type RsmevState<S> = State<Arc<Rsmev<S>>>;
async fn send_request<S: Service>(
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
    HeaderNodeId(node_id): HeaderNodeId,
    Json(request): Json<SendRequest>,
) -> String {
    let task_id = state
        .push_task(entrypoint_id, node_id, request.message)
        .await;

    task_id.to_string()
}

async fn get_request<S: Service>(
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
    HeaderNodeId(node_id): HeaderNodeId,
) -> String {
    let _ = state.pop_task(entrypoint_id, node_id).await.unwrap();
    "Response".to_string()
}

async fn confirm_request<S: Service>(
    State(state): RsmevState<S>,
    Path((entrypoint_id, request_id)): Path<(Uuid, Uuid)>,
    HeaderNodeId(node_id): HeaderNodeId,
) -> String {
    state.confirm_task(entrypoint_id, node_id, request_id).await;

    "Ok".to_string()
}

struct Rsmev<S: Service> {
    service: Arc<S>,
    clients: DashMap<Uuid, Client<S>>,
}

impl<S: Service> Rsmev<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            clients: DashMap::new(),
        }
    }

    pub async fn push_task(
        &self,
        entrypoint_id: Uuid,
        node_id: Option<String>,
        message: Message,
    ) -> Uuid {
        self.get_client(entrypoint_id)
            .push_task(node_id, message)
            .await
    }

    pub async fn pop_task(
        &self,
        entrypoint_id: Uuid,
        node_id: Option<String>,
    ) -> Option<crate::service::Result<S>> {
        self.get_client(entrypoint_id).pop_task(node_id).await
    }

    pub async fn confirm_task(
        &self,
        entrypoint_id: Uuid,
        node_id: Option<String>,
        request_id: Uuid,
    ) {
        self.get_client(entrypoint_id)
            .confirm_task(node_id, &request_id)
            .await;
    }

    pub fn get_client(
        &self,
        entrypoint_id: Uuid,
    ) -> dashmap::mapref::one::RefMut<'_, Uuid, Client<S>> {
        self.clients
            .entry(entrypoint_id)
            .or_insert(Client::new(self.service.clone()))
    }
}
