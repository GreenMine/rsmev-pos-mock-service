use std::sync::Arc;

use super::{body::Body, client::Client, extractor::HeaderNodeId, handler_service::HandlerService};
use crate::service::Service;

use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use dashmap::DashMap;
pub use tokio::net::TcpListener;
use uuid::Uuid;

pub async fn serve<S: Service>(listener: TcpListener, service: S) -> Result<(), std::io::Error> {
    let state = Arc::new(Rsmev::new(service));

    let rsmev_routes = Router::new()
        .route("/sendrequest", post(send_request))
        .route("/getresponse", post(get_response))
        .route("/confirmprocessing/:request_id", post(confirm_request))
        .with_state(state);

    let routes = Router::new().nest("/api/smev/:entrypoint_id", rsmev_routes);
    axum::serve(listener, routes).await
}

#[derive(serde::Deserialize, Debug)]
struct SendRequest {
    #[serde(flatten)]
    body: Body,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SendResponse {
    request_id: Uuid,
}

type RsmevState<S> = State<Arc<Rsmev<S>>>;
async fn send_request<S: Service>(
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
    HeaderNodeId(node_id): HeaderNodeId,
    Json(request): Json<SendRequest>,
) -> Json<SendResponse> {
    let task_id = state.push_task(entrypoint_id, node_id, request.body).await;

    Json(SendResponse {
        request_id: task_id,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GetResponse {
    rec_id: Uuid,
    request_id: Uuid,
    message_id: Uuid,
    #[serde(flatten)]
    body: Body,
}

async fn get_response<S: Service>(
    State(state): RsmevState<S>,
    Path(entrypoint_id): Path<Uuid>,
    HeaderNodeId(node_id): HeaderNodeId,
) -> Json<GetResponse> {
    let (request_id, body) = state.pop_task(entrypoint_id, node_id).await.unwrap();

    Json(GetResponse {
        rec_id: request_id,
        request_id,
        message_id: Uuid::new_v4(),
        body: body.unwrap(),
    })
}

async fn confirm_request<S: Service>(
    State(state): RsmevState<S>,
    Path((entrypoint_id, request_id)): Path<(Uuid, Uuid)>,
    HeaderNodeId(node_id): HeaderNodeId,
) -> () {
    state.confirm_task(entrypoint_id, node_id, request_id).await;
}

struct Rsmev<S: Service> {
    service: Arc<HandlerService<S>>,
    clients: DashMap<Uuid, Client<S>>,
}

impl<S: Service> Rsmev<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(HandlerService::new(service)),
            clients: DashMap::new(),
        }
    }

    pub async fn push_task(
        &self,
        entrypoint_id: Uuid,
        node_id: Option<String>,
        body: Body,
    ) -> Uuid {
        self.get_client(entrypoint_id)
            .push_task(node_id, body)
            .await
    }

    pub async fn pop_task(
        &self,
        entrypoint_id: Uuid,
        node_id: Option<String>,
    ) -> Option<(Uuid, crate::rsmev::Result<S>)> {
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
