use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    Json,
};
use redis::AsyncCommands;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::ApplicationState;

pub async fn print_redis_state(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Json<Value> {
    Json(get_state(state, id).await)
}

pub async fn get_state(state: Arc<ApplicationState>, id: Uuid) -> Value {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let pattern = format!("status:{}:*:info", id);
    let keys: Vec<String> = conn.keys(pattern).await.unwrap();

    let mut result = serde_json::Map::new();

    for key in keys {
        if let Ok(service_data) = conn
            .hgetall::<String, HashMap<String, String>>(key.clone())
            .await
        {
            // Insert the service data into the JSON map
            result.insert(key, json!(service_data));
        }
    }

    // Return as a JSON object
    Value::Object(result)
}

pub async fn get_status_report3(
    state: Arc<ApplicationState>,
    id: Uuid,
) -> HashMap<String, serde_json::Value> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let worker_info = get_worker_info(&mut conn, id).await;

    let mut status_report = HashMap::new();

    // Combine worker info
    for (key, data) in worker_info {
        status_report.insert(key, serde_json::json!(data));
    }

    status_report
}

async fn get_worker_info(
    conn: &mut redis::aio::MultiplexedConnection,
    id: Uuid,
) -> HashMap<String, HashMap<String, serde_json::Value>> {
    let pattern = format!("status:{}:*:*:*:info", id);
    let keys: Vec<String> = conn.keys(&pattern).await.unwrap();

    let completed_pattern = format!("status:{}:*:info", id);
    let completed_keys: Vec<String> = conn.keys(&completed_pattern).await.unwrap();

    let keys = keys
        .into_iter()
        .chain(completed_keys.into_iter())
        .collect::<Vec<String>>();

    let mut worker_info = HashMap::new();

    for key in keys {
        let service_data: HashMap<String, String> = match conn.hgetall(&key).await {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to HGETALL key {}: {}", key, err);
                continue;
            }
        };

        let mut deserialized_data = HashMap::new();
        for (field, data_str) in service_data {
            let deserialized: serde_json::Value = serde_json::json!(data_str);
            deserialized_data.insert(field, deserialized);
        }

        worker_info.insert(key, deserialized_data);
    }

    worker_info
}
