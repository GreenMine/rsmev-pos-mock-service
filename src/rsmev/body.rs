use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct File {
    name: String,
    url: String,
    #[serde(rename = "signaturePKCS7")]
    signature: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Body {
    pub xml: EncodedXml,
    pub files: Vec<File>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct EncodedXml {
    content: String,
}

#[derive(Debug)]
pub struct Error;

impl EncodedXml {
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    pub fn deserialize<'de, T: Deserialize<'de>>(&self) -> Result<T, Error> {
        let decoded = BASE64_STANDARD.decode(&self.content).map_err(|_| Error)?;
        let cursor = std::io::Cursor::new(decoded);

        let mut deserializer = quick_xml::de::Deserializer::from_reader(cursor);

        T::deserialize(&mut deserializer).map_err(|_| Error)
    }

    pub fn serialize<T: Serialize>(content: &T) -> Result<Self, Error> {
        let serialized = quick_xml::se::to_string(content).map_err(|_| Error)?;

        Ok(Self::new(BASE64_STANDARD.encode(&serialized)))
    }
}
