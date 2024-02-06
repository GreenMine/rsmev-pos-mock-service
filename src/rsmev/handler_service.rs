use crate::service::{Message, Service};

use super::body::Body as RsmevBody;

pub struct HandlerService<S> {
    service: S,
}

impl<S: Service> HandlerService<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn handle(&self, body: RsmevBody) -> crate::rsmev::Result<S> {
        let content = Message::try_from(body).unwrap();
        let response = self.service.handle(content).await;

        // FIXME: weird
        Ok(response.map(RsmevBody::try_from)?.unwrap())
    }
}
