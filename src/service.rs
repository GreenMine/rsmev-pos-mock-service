use crate::rsmev::Request;

pub type Result<S> = std::result::Result<<S as Service>::Response, <S as Service>::Error>;

// TODO: maybe just add a associated type Response(which may be a result, if it can be failed)
pub trait Service: Send + Sync + 'static {
    type Response: serde::Serialize + Clone + Send + Sync;
    type Error: std::error::Error + Clone + Send + Sync;

    fn handle(&self, content: Request) -> impl std::future::Future<Output = Result<Self>> + Send;
}
