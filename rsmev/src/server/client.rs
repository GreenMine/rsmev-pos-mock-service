use std::sync::Arc;

use super::body::Body;
use super::handler_service::HandlerService;
use crate::confirm_queue::{ConfirmQueue, KeyGenerator, UuidKey};
use crate::service::Service;

use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

type ChannelTransferType = (Option<NodeId>, Uuid, Body);
type Queue<T> = ConfirmQueue<T, QUEUE_TTL, UuidKey>;
type QueueKey = Uuid;

const CHANNEL_BUFFER_SIZE: usize = 256;
const QUEUE_TTL: u64 = 1 * 10 * 1000;

pub struct Client {
    nodes: Arc<Nodes<Body>>,
    tx: mpsc::Sender<ChannelTransferType>,
}

const BASE_NODE_ID: &str = "master";

impl Client {
    pub fn new<S: Service>(service: Arc<HandlerService<S>>) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let nodes = Arc::new(Nodes::new());

        Self::spawn_handler(service, nodes.clone(), rx);
        Self { nodes, tx }
    }

    pub async fn push_task(&self, node_id: Option<NodeId>, body: Body) -> QueueKey {
        let key = UuidKey::generate();
        // TODO: throw the error up
        let _ = self.tx.send((node_id, key, body)).await;

        key
    }

    pub async fn pop_task(&self, node_id: Option<NodeId>) -> Option<(QueueKey, Body)> {
        self.nodes
            .node(node_id)
            .take()
            .map(|(id, result)| (*id, result.clone()))
    }

    // TODO: remove files after confirm
    pub async fn confirm_task(&self, node_id: Option<NodeId>, task_id: &QueueKey) {
        self.nodes.node(node_id).confirm(task_id);
    }

    fn spawn_handler<S: Service>(
        service: Arc<HandlerService<S>>,
        nodes: Arc<Nodes<Body>>,
        mut rx: mpsc::Receiver<ChannelTransferType>,
    ) {
        tokio::spawn(async move {
            while let Some((node_id, key, request)) = rx.recv().await {
                let response = service.handle(request).await;
                nodes.node(node_id).add_with_key(key, response);
            }
        });
    }
}

type NodeId = String;
struct Nodes<T> {
    inner: DashMap<NodeId, Queue<T>>,
}

impl<T> Nodes<T> {
    pub fn new() -> Self {
        Nodes {
            inner: DashMap::new(),
        }
    }

    pub fn node(&self, name: Option<String>) -> dashmap::mapref::one::RefMut<'_, NodeId, Queue<T>> {
        self.inner
            .entry(name.unwrap_or(BASE_NODE_ID.to_string()))
            .or_insert(Queue::new())
    }
}
