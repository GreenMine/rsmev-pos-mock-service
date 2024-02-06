use std::io::BufReader;

use super::body::Body as RsmevBody;
use crate::service::{Message, Service};

use async_ftp::FtpStream;
use tokio::net::ToSocketAddrs;

pub struct HandlerService<S> {
    service: S,
}

impl<S: Service> HandlerService<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn handle(&self, body: RsmevBody) -> crate::rsmev::Result<S> {
        let client = self.ftp_client().await;
        // let content = Message::try_from(body).unwrap();
        // let response = self.service.handle(content).await;
        //
        // // FIXME: weird
        // Ok(response.map(RsmevBody::try_from)?.unwrap())

        unimplemented!()
    }

    async fn ftp_client(&self) -> FtpClient {
        FtpClient::connect("localhost:21", "admin", "12345678").await
    }
}

struct FtpClient {
    stream: FtpStream,
}

impl FtpClient {
    pub async fn connect(host: impl ToSocketAddrs, login: &str, password: &str) -> Self {
        // FIXME: INSECURE!!!
        let mut stream = FtpStream::connect(host).await.unwrap();
        let _ = stream.login(login, password).await.unwrap();

        Self { stream }
    }

    pub async fn download_file(&mut self, file: &str) -> Vec<std::fs::File> {
        use std::io::Write;
        let file = std::fs::File::create("asdf").unwrap().write();

        self.stream.simple_retr(file_name)
        unimplemented!()
    }

    pub async fn upload_file(&mut self) {}
}
