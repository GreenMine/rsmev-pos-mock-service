mod confirm_queue;
use confirm_queue::ConfirmQueue;

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
pub use tokio::{net::TcpListener, sync::mpsc};

use crate::Service;

use self::confirm_queue::{KeyGenerator, UuidKey};

pub async fn serve<S: Service>(listener: TcpListener, service: S) -> Result<(), std::io::Error> {
    let state = Arc::new(Rsmev::new(service));

    let app = Router::new()
        .route("/send", get(send_request))
        .route("/get", get(get_request))
        .route("/confirm/:request_id", get(confirm_request))
        .with_state(state);
    axum::serve(listener, app).await
}

type RsmevState<S> = State<Arc<Rsmev<S>>>;
async fn send_request<S: Service>(State(state): RsmevState<S>) -> String {
    state.push_task("asdf".to_string()).await
}

async fn get_request<S: Service>(State(state): RsmevState<S>) -> String {
    state.pop_task().await.unwrap_or("None".to_string())
}

async fn confirm_request<S: Service>(
    State(state): RsmevState<S>,
    Path(request_id): Path<String>,
) -> String {
    state.confirm_task(request_id).await;

    "Ok".to_string()
}

struct Rsmev<S> {
    service: Arc<S>,
    requests_sender: mpsc::Sender<(String, String)>,

    queue: Arc<Mutex<ConfirmQueue<String>>>,
}

impl<S: Service> Rsmev<S> {
    pub fn new(service: S) -> Self {
        let queue = Arc::new(Mutex::new(ConfirmQueue::new()));
        let service = Arc::new(service);

        let (sender, mut receiver): (
            mpsc::Sender<(String, String)>,
            mpsc::Receiver<(String, String)>,
        ) = mpsc::channel(1024);
        tokio::spawn({
            let q = queue.clone();
            let s = service.clone();
            async move {
                while let Some((k, content)) = receiver.recv().await {
                    let r = s.handle(&content).await;
                    q.lock().await.add_with_key(k, r);
                }
            }
        });

        Self {
            service,
            requests_sender: sender,
            queue,
        }
    }

    pub async fn push_task(&self, task: String) -> String {
        let key = UuidKey::generate();
        self.requests_sender
            .send((key.clone(), task))
            .await
            .unwrap();

        key
    }

    pub async fn pop_task(&self) -> Option<String> {
        self.queue.lock().await.take().map(|i| i.1.clone())
    }

    pub async fn confirm_task(&self, key: String) {
        self.queue.lock().await.confirm(&key);
    }
}
