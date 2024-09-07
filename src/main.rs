use std::sync::Arc;

use axum::{routing::get, Router};
use redis::Client as RedisClient;
use rust_kafka_streams::{
    consumer::{consume_kafka_messages, init_stream_consumer},
    handler::print_redis_state,
    models::ApplicationState,
};
use tokio::task::{self, JoinHandle};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let consumer_topic = vec!["test-status-topic"];
    let consumer = init_stream_consumer("localhost:29092", consumer_topic);
    let redis_client = RedisClient::open("redis://127.0.0.1/").unwrap();
    let state = Arc::new(ApplicationState { redis_client });
    let state_clone = Arc::clone(&state);

    // Spawn Kafka task with error handling inside the task
    let kafka_task: JoinHandle<()> = tokio::spawn(async move {
        if let Err(e) = consume_kafka_messages(state_clone, consumer, None).await {
            eprintln!("Error in Kafka task: {:?}", e);
        }
    });

    let api_task: JoinHandle<()> = task::spawn({
        let state = Arc::clone(&state);
        async move {
            let app = Router::new()
                .route("/state/:id", get(print_redis_state))
                .with_state(state);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!("Listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        }
    });

    tokio::try_join!(kafka_task, api_task).unwrap();

    Ok(())
}
