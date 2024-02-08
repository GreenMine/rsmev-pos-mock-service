use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

const NODE_ID_HEADER_NAME: &'static str = "node_id";
pub struct HeaderNodeId(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for HeaderNodeId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(header) = parts.headers.get(NODE_ID_HEADER_NAME) {
            let value = header
                .to_str()
                .map_err(|_| (StatusCode::BAD_REQUEST, "`node_id` header is not a string"))?;
            Ok(HeaderNodeId(Some(value.to_string())))
        } else {
            Ok(HeaderNodeId(None))
        }
    }
}
