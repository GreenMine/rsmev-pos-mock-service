use std::env;

use uuid::Uuid;

use super::body::{Body as RsmevBody, EncodedXml, File as RsmevFile};
use crate::service::{Message, Service};

pub struct HandlerService<S> {
    service: S,
}

const BASE_FILE_DIR: &'static str = "./ftp_data";

impl<S: Service> HandlerService<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn handle(&self, body: RsmevBody) -> crate::server::Result<S> {
        let content = Self::to_message(body);

        let response = self.service.handle(content).await;

        response.map(Self::to_rsmev_body)
    }

    pub(crate) fn to_message(body: RsmevBody) -> Message<S::Request> {
        let current_dir = env::current_dir().unwrap();

        let RsmevBody { files, xml } = body;

        let files = files
            .into_iter()
            .map(|f| {
                let mut file_path = current_dir.clone();
                file_path.push(BASE_FILE_DIR);
                file_path.push(f.url);

                if let Ok(_) = std::fs::metadata(&file_path) {
                    Ok(file_path)
                } else {
                    Err(())
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        Message {
            content: xml.deserialize().unwrap(),
            files,
        }
    }

    pub(crate) fn to_rsmev_body(message: Message<S::Response>) -> RsmevBody {
        let current_dir = env::current_dir().unwrap();

        let Message { content, files } = message;

        let files = files
            .into_iter()
            .map(|file| {
                let mut new_path = current_dir.clone();
                new_path.push(BASE_FILE_DIR);

                let path_id = Uuid::new_v4().to_string();
                new_path.push(&path_id);

                std::fs::create_dir(&new_path).unwrap();
                let file_name = file.file_name().unwrap().to_string_lossy().to_string();
                new_path.push(&file_name);

                std::fs::rename(file, new_path).unwrap();
                RsmevFile {
                    name: file_name.to_string(),
                    url: format!("/{path_id}/{file_name}"),
                    signature: None,
                }
            })
            .collect::<Vec<_>>();

        RsmevBody {
            xml: EncodedXml::serialize(&content).unwrap(),
            files,
        }
    }
}
