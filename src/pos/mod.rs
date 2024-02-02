use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

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

impl crate::service::Service for Pos {
    type Error = Infallible;
    type Response = String;

    async fn handle(&self, _content: crate::rsmev::Request) -> crate::service::Result<Self> {
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;

        Ok(format!("pos{}", self.next()))
    }
}
