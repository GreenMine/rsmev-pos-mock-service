pub trait Service: Send + Sync + 'static {
    fn handle(&self, content: &str) -> impl std::future::Future<Output = String> + Send;
}
