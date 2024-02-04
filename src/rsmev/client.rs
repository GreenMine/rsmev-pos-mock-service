use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::service::{Result as ServiceResult, Service};

use super::Message;
use crate::confirm_queue::{ConfirmQueue, KeyGenerator, UuidKey};
use dashmap::DashMap;

type ChannelTransferType = (Option<NodeId>, Uuid, Message);
type Queue<T> = ConfirmQueue<T, QUEUE_TTL, UuidKey>;
type QueueKey = Uuid;

type ServiceNodes<S> = Nodes<ServiceResult<S>>;

const CHANNEL_BUFFER_SIZE: usize = 256;
const QUEUE_TTL: u64 = 1 * 10 * 1000;

pub struct Client<S: Service> {
    nodes: Arc<ServiceNodes<S>>,
    tx: mpsc::Sender<ChannelTransferType>,
}

const BASE_NODE_ID: &'static str = "master";

impl<S: Service> Client<S> {
    pub fn new(service: Arc<S>) -> Self {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let nodes = Arc::new(Nodes::new());

        Self::spawn_handler(service, nodes.clone(), rx);
        Self { nodes, tx }
    }

    pub async fn push_task(&self, node_id: Option<NodeId>, message: Message) -> QueueKey {
        let key = UuidKey::generate();
        // TODO: throw the error up
        let _ = self.tx.send((node_id, key.clone(), message)).await;

        key
    }

    pub async fn pop_task(&self, node_id: Option<NodeId>) -> Option<ServiceResult<S>> {
        self.nodes.node(node_id).take().map(|qi| qi.1.clone())
    }

    pub async fn confirm_task(&self, node_id: Option<NodeId>, task_id: &QueueKey) {
        self.nodes.node(node_id).confirm(task_id);
    }

    fn spawn_handler(
        service: Arc<S>,
        nodes: Arc<ServiceNodes<S>>,
        mut rx: mpsc::Receiver<ChannelTransferType>,
    ) {
        tokio::spawn(async move {
            while let Some((node_id, key, request)) = rx.recv().await {
                let response = service.process(request).await;
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
