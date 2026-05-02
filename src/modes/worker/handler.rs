use async_trait::async_trait;
use rdkafka::message::{BorrowedMessage, Message};

/// Owned copy of a consumed record — handlers stay decoupled from `rdkafka` types.
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub partition: i32,
    pub offset: i64,
    pub key: Option<Vec<u8>>,
    pub payload: Vec<u8>,
}

impl InboundMessage {
    pub fn from_kafka(m: &BorrowedMessage<'_>) -> Self {
        Self {
            partition: m.partition(),
            offset: m.offset(),
            key: m.key().map(|k| k.to_vec()),
            payload: m.payload().map(|p| p.to_vec()).unwrap_or_default(),
        }
    }
}

/// Process one message — swap implementations for domain logic (e.g. deserialize JSON, call services).
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(
        &self,
        msg: InboundMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Default: structured logging (replace with your handler via `run_with_handler` pattern when needed).
pub struct LogMessageHandler;

#[async_trait]
impl MessageHandler for LogMessageHandler {
    async fn handle(
        &self,
        msg: InboundMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key_dbg = msg.key.as_ref().map(|k| format!("{k:?}"));
        let payload = if msg.payload.is_empty() {
            "<empty>".to_string()
        } else if let Ok(s) = std::str::from_utf8(&msg.payload) {
            s.to_string()
        } else {
            format!("<{} bytes binary>", msg.payload.len())
        };
        tracing::info!(
            mode = "worker",
            partition = msg.partition,
            offset = msg.offset,
            ?key_dbg,
            %payload,
            "message received"
        );
        Ok(())
    }
}
