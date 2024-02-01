use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::Service;

use super::{Request, Response};
use crate::confirm_queue::{ConfirmQueue, KeyGenerator, UuidKey};
use dashmap::DashMap;

type ChannelTransferType = (Option<NodeId>, Uuid, Request);
type Queue = ConfirmQueue<Response, QUEUE_TTL, UuidKey>;
type QueueKey = Uuid;

const CHANNEL_BUFFER_SIZE: usize = 256;
const QUEUE_TTL: u64 = 1 * 10 * 1000;

pub struct Client {
    nodes: Arc<Nodes>,
    tx: mpsc::Sender<ChannelTransferType>,
}

const BASE_NODE_ID: &'static str = "master";

impl Client {
    pub fn new<S: Service>(service: Arc<S>) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let nodes = Arc::new(Nodes::new());

        Self::spawn_handler(service, nodes.clone(), rx);
        Self { nodes, tx }
    }

    pub async fn push_task(&self, node_id: Option<NodeId>, request: Request) -> QueueKey {
        let key = UuidKey::generate();
        // TODO: throw the error up
        let _ = self.tx.send((node_id, key.clone(), request)).await;

        key
    }

    pub async fn pop_task(&self, node_id: Option<NodeId>) -> Option<Response> {
        self.nodes.node(node_id).take().map(|qi| qi.1.clone())
    }

    pub async fn confirm_task(&self, node_id: Option<NodeId>, task_id: &QueueKey) {
        self.nodes.node(node_id).confirm(task_id);
    }

    fn spawn_handler<S: Service>(
        service: Arc<S>,
        nodes: Arc<Nodes>,
        mut rx: mpsc::Receiver<ChannelTransferType>,
    ) {
        tokio::spawn(async move {
            while let Some((node_id, key, request)) = rx.recv().await {
                let response = service.handle(&request).await;
                nodes.node(node_id).add_with_key(key, response);
            }
        });
    }
}

type NodeId = String;
struct Nodes {
    inner: DashMap<NodeId, Queue>,
}

impl Nodes {
    pub fn new() -> Self {
        Nodes {
            inner: DashMap::new(),
        }
    }

    pub fn node(&self, name: Option<String>) -> dashmap::mapref::one::RefMut<'_, NodeId, Queue> {
        self.inner
            .entry(name.unwrap_or(BASE_NODE_ID.to_string()))
            .or_insert(Queue::new())
    }
}
