use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Pos {
    i: AtomicUsize,
}

impl Pos {
    pub const fn new() -> Self {
        Self {
            i: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> usize {
        self.i.fetch_add(1, Ordering::Relaxed)
    }
}

impl crate::Service for Pos {
    async fn handle(&self, _content: &str) -> String {
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

        format!("pos{}", self.next())
    }
}
