use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct File {
    pub name: String,
    pub url: String,
    #[serde(rename = "signaturePKCS7")]
    pub signature: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Body {
    pub xml: EncodedXml,
    #[serde(default)]
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
        println!("XML: {}", unsafe {
            std::str::from_utf8_unchecked(&decoded)
        });
        let cursor = std::io::Cursor::new(decoded);

        let mut deserializer = quick_xml::de::Deserializer::from_reader(cursor);

        T::deserialize(&mut deserializer).map_err(|e| {
            tracing::error!(err = ?e);
            Error
        })
    }

    pub fn serialize<T: Serialize>(content: &T) -> Result<Self, Error> {
        let serialized = quick_xml::se::to_string(content).map_err(|_| Error)?;

        println!("Serialized:\n{serialized}");

        Ok(Self::new(BASE64_STANDARD.encode(&serialized)))
    }
}
