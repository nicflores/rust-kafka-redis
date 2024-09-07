use std::sync::Arc;

use futures::stream::StreamExt;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Message;
use rdkafka::ClientConfig;

use crate::models::{ApplicationState, StatusMessage};
use crate::processor::process_status_message;

pub fn init_stream_consumer(bootstrap_server: &str, topics: Vec<&str>) -> StreamConsumer {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_server)
        .set("auto.offset.reset", "earliest")
        .set("group.id", "test-group")
        .set("socket.timeout.ms", "4000")
        .create()
        .expect("Failed to create consumer.");

    consumer
        .subscribe(&topics)
        .expect("Couldn't subscirbe to topics.");

    consumer
}

pub async fn consume_kafka_messages(
    state: Arc<ApplicationState>,
    consumer: StreamConsumer,
    message_limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut processed_messages = 0;
    let mut stream = consumer.stream();

    while let Some(message) = stream.next().await {
        match message {
            Ok(m) => {
                if let Some(payload) = m.payload() {
                    let payload_str = String::from_utf8_lossy(payload);
                    let status_message: StatusMessage = serde_json::from_str(&payload_str).unwrap();
                    process_status_message(&state, status_message).await?;
                }
                processed_messages += 1;
                if let Some(limit) = message_limit {
                    if processed_messages >= limit {
                        println!("Reached message limit of {}", limit);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error consuming Kafka message: {:?}", e);
            }
        }
    }
    Ok(())
}
