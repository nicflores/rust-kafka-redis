#[cfg(test)]
mod integration_tests {
    use chrono::Utc;
    use rust_kafka_streams::consumer::consume_kafka_messages;
    use rust_kafka_streams::consumer::init_stream_consumer;
    use rust_kafka_streams::handler::get_status_report3;
    use rust_kafka_streams::models::ApplicationState;
    use rust_kafka_streams::producer::init_producer;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_kafka_consumer_state_processing() {
        let producer = Arc::new(init_producer("localhost:29092"));
        let producer_topic = "test-status-topic";

        let consumer_topic = vec!["test-status-topic"];

        let worker_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let client_id = 1234;

        let messages = vec![
            json!({
                "service": "worker",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "started",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:00:00Z",
                "client_id": client_id,
                "payload": {}
            }),
            json!({
                "service": "worker",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "message_created",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:01:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "security"}
            }),
            json!({
                "service": "worker",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "message_created",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:02:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "index"}
            }),
            json!({
                "service": "worker",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "done",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:03:00Z",
                "client_id": client_id,
                "payload": {}
            }),
            json!({
                "service": "feeder",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "rules_applied",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:04:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "index"}
            }),
            json!({
                "service": "feeder",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "rules_applied",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:05:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "index"}
            }),
            json!({
                "service": "warehouser",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "message_stored",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:06:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "index"}
            }),
            json!({
                "service": "warehouser",
                "worker_id": worker_id.to_string(),
                "vendor": "bloomberg",
                "action": "message_stored",
                "id": run_id.to_string(),
                "timestamp": "2024-01-01T00:07:00Z",
                "client_id": client_id,
                "payload": {"count": 1, "type": "index"}
            }),
        ];

        let messages_len = messages.len();
        println!("Sending {} messages to Kafka", messages_len);
        for msg in messages {
            producer
                .send(
                    rdkafka::producer::FutureRecord::to(&producer_topic)
                        .payload(&serde_json::to_string(&msg).unwrap())
                        .key("processed_message")
                        .timestamp(Utc::now().timestamp_millis()),
                    Duration::from_secs(0),
                )
                .await
                .expect("Failed to send message");
        }

        let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let consumer = init_stream_consumer("localhost:29092", consumer_topic);
        let state = Arc::new(ApplicationState { redis_client });

        consume_kafka_messages(Arc::clone(&state), consumer, Some(messages_len))
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let state_data = get_status_report3(Arc::clone(&state), run_id).await;

        println!("{}", serde_json::to_string_pretty(&state_data).unwrap());
    }
}
