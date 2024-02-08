use std::sync::Arc;

use super::{body::Body, client::Client, extractor::HeaderNodeId, handler_service::HandlerService};
use crate::service::Service;

use axum::{
    extract::{Path, State},
    http::StatusCode,
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
    #[cfg(feature = "tracing_requests")]
    let router = router.layer(axum::middleware::from_fn(middleware::print_request_body));

    axum::serve(listener, routes).await
}

#[cfg(feature = "tracing_requests")]
mod middleware {
    use axum::{
        async_trait,
        body::{Body, Bytes},
        extract::{FromRequest, Request},
        http::StatusCode,
        middleware::Next,
        response::{IntoResponse, Response},
    };
    use http_body_util::BodyExt;
    // middleware that shows how to consume the request body upfront
    pub(crate) async fn print_request_body(
        request: Request,
        next: Next,
    ) -> Result<impl IntoResponse, Response> {
        let request = buffer_request_body(request).await?;

        Ok(next.run(request).await)
    }

    // the trick is to take the request apart, buffer the body, do what you need to do, then put
    // the request back together
    async fn buffer_request_body(request: Request) -> Result<Request, Response> {
        let (parts, body) = request.into_parts();

        // this wont work if the body is an long running stream
        let bytes = body
            .collect()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
            .to_bytes();

        do_thing_with_request_body(bytes.clone());

        Ok(Request::from_parts(parts, Body::from(bytes)))
    }

    fn do_thing_with_request_body(bytes: Bytes) {
        tracing::debug!(body = ?bytes);
    }

    async fn handler(BufferRequestBody(body): BufferRequestBody) {
        tracing::debug!(?body, "handler received body");
    }

    // extractor that shows how to consume the request body upfront
    struct BufferRequestBody(Bytes);

    // we must implement `FromRequest` (and not `FromRequestParts`) to consume the body
    #[async_trait]
    impl<S> FromRequest<S> for BufferRequestBody
    where
        S: Send + Sync,
    {
        type Rejection = Response;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
            let body = Bytes::from_request(req, state)
                .await
                .map_err(|err| err.into_response())?;

            do_thing_with_request_body(body.clone());

            Ok(Self(body))
        }
    }
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
) -> (StatusCode, Json<Option<GetResponse>>) {
    if let Some((request_id, body)) = state.pop_task(entrypoint_id, node_id).await {
        (
            StatusCode::OK,
            Json(Some(GetResponse {
                rec_id: request_id,
                request_id,
                message_id: Uuid::new_v4(),
                body: body.unwrap(),
            })),
        )
    } else {
        (StatusCode::NOT_FOUND, Json(None))
    }
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
    ) -> Option<(Uuid, crate::server::Result<S>)> {
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
