mod handler;
mod kafka;

use std::time::Duration;

pub use handler::{InboundMessage, LogMessageHandler, MessageHandler};

use futures::StreamExt;
use rdkafka::consumer::Consumer;

use crate::config::WorkerSettings;

const POLL_TIMEOUT: Duration = Duration::from_secs(30);

/// Run the worker with the default [`LogMessageHandler`].
pub async fn run(settings: WorkerSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_with_handler(settings, LogMessageHandler).await
}

/// Run the worker with a custom [`MessageHandler`] (e.g. JSON → service calls).
pub async fn run_with_handler<H: MessageHandler + 'static>(
    settings: WorkerSettings,
    handler: H,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        mode = "worker",
        brokers = %settings.kafka_brokers,
        topic = %settings.kafka_topic,
        group_id = %settings.kafka_group_id,
        "starting Kafka consumer"
    );

    let consumer = kafka::stream_consumer(&settings)?;
    consumer.subscribe(&[&settings.kafka_topic])?;

    let mut stream = consumer.stream();
    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                tracing::info!(mode = "worker", "shutting down consumer");
                break;
            }
            msg = tokio::time::timeout(POLL_TIMEOUT, stream.next()) => {
                match msg {
                    Ok(Some(Ok(m))) => {
                        let inbound = InboundMessage::from_kafka(&m);
                        if let Err(e) = handler.handle(inbound).await {
                            tracing::warn!(mode = "worker", error = %e, "message handler error");
                        }
                    }
                    Ok(Some(Err(e))) => tracing::warn!(mode = "worker", error = %e, "kafka receive error"),
                    Ok(None) => tracing::debug!(mode = "worker", "stream ended"),
                    Err(_) => tracing::trace!(mode = "worker", "poll timeout (no message)"),
                }
            }
        }
    }

    Ok(())
}
