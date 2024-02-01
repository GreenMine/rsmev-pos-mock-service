mod client;
use client::Client;

mod extractor;
use extractor::HeaderNodeId;

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use std::sync::Arc;
pub use tokio::net::TcpListener;
use uuid::Uuid;

use dashmap::DashMap;

use crate::Service;

pub type Request = String;
pub type Response = String;

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

type RsmevState<S> = State<Arc<Rsmev<S>>>;
async fn send_request<S: Service>(
    HeaderNodeId(node_id): HeaderNodeId,
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
) -> String {
    tracing::info!("Node id: {:?}", node_id);
    let task_id = state.push_task(entrypoint_id, "asdf".to_string()).await;

    task_id.to_string()
}

async fn get_request<S: Service>(
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
) -> String {
    state
        .pop_task(entrypoint_id)
        .await
        .unwrap_or("None".to_string())
}

async fn confirm_request<S: Service>(
    State(state): RsmevState<S>,
    Path((entrypoint_id, request_id)): Path<(Uuid, Uuid)>,
) -> String {
    state.confirm_task(entrypoint_id, request_id).await;

    "Ok".to_string()
}

struct Rsmev<S> {
    service: Arc<S>,
    clients: DashMap<Uuid, Client>,
}

impl<S: Service> Rsmev<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            clients: DashMap::new(),
        }
    }

    pub async fn push_task(&self, entrypoint_id: Uuid, request: Request) -> Uuid {
        self.get_client(entrypoint_id).push_task(request).await
    }

    pub async fn pop_task(&self, entrypoint_id: Uuid) -> Option<Response> {
        self.get_client(entrypoint_id).pop_task().await
    }

    pub async fn confirm_task(&self, entrypoint_id: Uuid, request_id: Uuid) {
        self.get_client(entrypoint_id)
            .confirm_task(&request_id)
            .await;
    }

    pub fn get_client(
        &self,
        entrypoint_id: Uuid,
    ) -> dashmap::mapref::one::RefMut<'_, Uuid, Client> {
        self.clients
            .entry(entrypoint_id)
            .or_insert(Client::new(self.service.clone()))
    }
}
