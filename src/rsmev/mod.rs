use axum::{extract::State, routing::get, Router};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
pub use tokio::{net::TcpListener, sync::mpsc};

use crate::Service;

pub async fn serve<S: Service>(listener: TcpListener, service: S) -> Result<(), std::io::Error> {
    let state = Arc::new(Rsmev::new(service));

    let app = Router::new()
        .route("/send", get(send_request))
        .route("/get", get(get_request))
        .with_state(state);
    axum::serve(listener, app).await
}

type RsmevState<S> = State<Arc<Rsmev<S>>>;
async fn send_request<S: Service>(State(state): RsmevState<S>) -> String {
    state.push_task("asdf".to_string()).await;

    "Ok".to_string()
}

async fn get_request<S: Service>(State(state): RsmevState<S>) -> String {
    state.pop_task().await.unwrap_or("None".to_string())
}

struct Rsmev<S> {
    service: Arc<S>,
    requests_sender: mpsc::Sender<String>,

    queue: Arc<Mutex<VecDeque<String>>>,
}

impl<S: Service> Rsmev<S> {
    pub fn new(service: S) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let service = Arc::new(service);

        let (sender, mut receiver): (mpsc::Sender<String>, mpsc::Receiver<String>) =
            mpsc::channel(1024);
        tokio::spawn({
            let q = queue.clone();
            let s = service.clone();
            async move {
                while let Some(v) = receiver.recv().await {
                    let r = s.handle(&v).await;
                    q.lock().await.push_back(r);
                }
            }
        });

        Self {
            service,
            requests_sender: sender,
            queue,
        }
    }

    pub async fn push_task(&self, task: String) {
        self.requests_sender.send(task).await.unwrap();
    }

    pub async fn pop_task(&self) -> Option<String> {
        self.queue.lock().await.pop_back()
    }
}
