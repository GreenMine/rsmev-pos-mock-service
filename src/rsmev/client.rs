use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::Service;

use super::{Request, Response};
use crate::confirm_queue::{ConfirmQueue, KeyGenerator, UuidKey};

type ChannelTransferType = (Uuid, Request);
type Queue = ConfirmQueue<Response, QUEUE_TTL, UuidKey>;
type AsyncQueue = Arc<Mutex<Queue>>;
type QueueKey = Uuid;

const CHANNEL_BUFFER_SIZE: usize = 256;
const QUEUE_TTL: u64 = 1 * 10 * 1000;

pub struct Client {
    queue: AsyncQueue,
    tx: mpsc::Sender<ChannelTransferType>,
}

impl Client {
    pub fn new<S: Service>(service: Arc<S>) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let queue = Arc::new(Mutex::new(ConfirmQueue::new()));

        Self::spawn_handler(service, queue.clone(), rx);
        Self { queue, tx }
    }

    pub async fn push_task(&self, request: Request) -> QueueKey {
        let key = UuidKey::generate();
        self.tx.send((key.clone(), request)).await;

        key
    }

    pub async fn pop_task(&self) -> Option<Response> {
        self.queue().await.take().map(|qi| qi.1.clone())
    }

    pub async fn confirm_task(&self, task_id: &QueueKey) {
        self.queue().await.confirm(task_id);
    }

    // FIXME: just return impl Future<Item = ...>
    async fn queue(&self) -> tokio::sync::MutexGuard<Queue> {
        self.queue.lock().await
    }

    fn spawn_handler<S: Service>(
        service: Arc<S>,
        queue: AsyncQueue,
        mut rx: mpsc::Receiver<ChannelTransferType>,
    ) {
        tokio::spawn(async move {
            while let Some((k, content)) = rx.recv().await {
                let r = service.handle(&content).await;
                queue.lock().await.add_with_key(k, r);
            }
        });
    }
}
