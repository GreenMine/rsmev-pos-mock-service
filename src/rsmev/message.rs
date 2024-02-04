use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct File {
    name: String,
    url: String,
    #[serde(rename = "signaturePKCS7")]
    signature: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Message<C> {
    #[serde(
        rename = "xml",
        with = "content_serde",
        bound(deserialize = "C: Deserialize<'de>", serialize = "C: Serialize")
    )]
    content: C,
    files: Vec<File>,
}

mod content_serde {
    use base64::prelude::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let decoded = BASE64_STANDARD
            .decode(string)
            .map_err(serde::de::Error::custom)?;
        let cursor = std::io::Cursor::new(decoded);

        let mut deserializer = quick_xml::de::Deserializer::from_reader(cursor);

        // TODO: transform this error to another error type which can contains current deserializer
        // error and another one(merge D::Error and quick_xml::Deserializer::Error)
        T::deserialize(&mut deserializer).map_err(serde::de::Error::custom)
    }

    pub fn serialize<C, S>(content: &C, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        C: Serialize,
    {
        let serialized = quick_xml::se::to_string(&content).map_err(serde::ser::Error::custom)?;
        let encoded = BASE64_STANDARD.encode(&serialized);

        serializer.serialize_str(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct AnyTag {
        #[serde(rename = "@one")]
        one: String,

        #[serde(rename = "@two")]
        two: String,
    }

    #[test]
    pub fn test_message_de() {
        let expected = Message {
            content: AnyTag {
                one: "one".to_string(),
                two: "two".to_string(),
            },
            files: Vec::new(),
        };

        let actual = r#"{"xml":"PEFueVRhZyBvbmU9Im9uZSIgdHdvPSJ0d28iLz4=","files":[]}"#;
        assert_eq!(expected, serde_json::from_str(actual).unwrap());
    }

    #[test]
    pub fn test_message_ser() {
        let expected = r#"{"xml":"PEFueVRhZyBvbmU9Im9uZSIgdHdvPSJ0d28iLz4=","files":[]}"#;
        let actual = Message {
            content: AnyTag {
                one: "one".to_string(),
                two: "two".to_string(),
            },
            files: Vec::new(),
        };

        assert_eq!(expected, serde_json::to_string(&actual).unwrap());
    }
}
