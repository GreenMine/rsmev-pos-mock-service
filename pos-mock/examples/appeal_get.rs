use pos_mock::{types, PosMock};
use rsmev::service::{Message, Service};

const DB: &'static str = "postgres://localuser:localpassword@localhost/rsmev_pos_mock";

#[tokio::main]
async fn main() {
    let pos = PosMock::new(DB).await;

    let content = Message {
        content: types::PosEdmsRequest {
            request: types::PosEdmsRequestTypes::AppealListRequest(types::AppealListRequest {
                client_id: uuid::uuid!("65c0e6ce-2219-4d15-8610-6eb9372fc58c"),
            }),
        },
        files: Vec::new(),
    };

    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    let result = pos.handle(content).await.unwrap();
    println!("Result: {:?}", result);
}
