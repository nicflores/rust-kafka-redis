use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    Json,
};
use redis::AsyncCommands;
use uuid::Uuid;

use crate::models::ApplicationState;

pub async fn print_redis_state(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Json<Vec<(String, HashMap<String, String>)>> {
    println!("Getting state for id: {}", id);
    Json(get_state(state, id).await)
}

pub async fn get_state(
    state: Arc<ApplicationState>,
    id: Uuid,
) -> Vec<(String, HashMap<String, String>)> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    //let keys: Vec<String> = conn.keys("*").await.unwrap();
    let pattern = format!("status:{}:*:info", id);
    let keys: Vec<String> = conn.keys(pattern).await.unwrap();
    let mut result = Vec::new();
    for key in keys {
        if let Ok(service_data) = conn
            .hgetall::<String, HashMap<String, String>>(key.clone())
            .await
        {
            result.push((key, service_data));
        }
        // if let Ok(value) = conn.hgetall(&key).await {
        //     result.push((key, value));
        // }
    }
    result
}

// pub async fn get_status_report(
//     state: Arc<ApplicationState>,
//     id: &str,
// ) -> HashMap<String, HashMap<String, serde_json::Value>> {
//     let mut conn = state
//         .redis_client
//         .get_multiplexed_async_connection()
//         .await
//         .unwrap();

//     // Use pattern matching to find all keys related to the given `id`
//     let pattern = format!("status:{}:*", id);
//     let keys: Vec<String> = conn.keys(pattern).await.unwrap();

//     let mut status_report = HashMap::new();

//     for key in keys {
//         let service_data: HashMap<String, String> = conn.hgetall(&key).await.unwrap();
//         let mut deserialized_data = HashMap::new();

//         for (field, data_str) in service_data {
//             let deserialized: serde_json::Value = match field.as_str() {
//                 "messages" | "errors" => serde_json::from_str(&data_str)
//                     .unwrap_or_else(|_| serde_json::json!({"error": "Failed to deserialize JSON"})),
//                 _ => serde_json::json!(data_str),
//             };
//             deserialized_data.insert(field, deserialized);
//         }

//         status_report.insert(key, deserialized_data);
//     }

//     status_report
// }

// pub async fn get_status_report2(
//     state: Arc<ApplicationState>,
//     id: Uuid,
// ) -> HashMap<String, HashMap<String, serde_json::Value>> {
//     let mut conn = state
//         .redis_client
//         .get_multiplexed_async_connection()
//         .await
//         .unwrap();

//     // Convert Uuid to a string and use it directly in the pattern
//     let pattern = format!("status:{}:*", id);
//     let keys: Vec<String> = conn.keys(&pattern).await.unwrap();

//     let mut status_report = HashMap::new();

//     for key in keys {
//         // Use the `type` command to get the type of the key
//         let key_type: RedisResult<String> =
//             redis::cmd("TYPE").arg(&key).query_async(&mut conn).await;

//         match key_type.unwrap_or_default().as_str() {
//             "hash" => {
//                 let service_data: HashMap<String, String> = match conn.hgetall(&key).await {
//                     Ok(data) => data,
//                     Err(err) => {
//                         eprintln!("Failed to HGETALL key {}: {}", key, err);
//                         continue;
//                     }
//                 };

//                 let mut deserialized_data = HashMap::new();

//                 for (field, data_str) in service_data {
//                     let deserialized: serde_json::Value = match field.as_str() {
//                         "messages" | "errors" => serde_json::from_str(&data_str).unwrap_or_else(
//                             |_| serde_json::json!({"error": "Failed to deserialize JSON"}),
//                         ),
//                         _ => serde_json::json!(data_str),
//                     };
//                     deserialized_data.insert(field, deserialized);
//                 }

//                 status_report.insert(key.clone(), deserialized_data);
//             }
//             "list" => {
//                 // Handle lists if necessary (e.g., for messages or errors)
//                 continue; // Optionally handle or skip lists
//             }
//             _ => {
//                 eprintln!("Ignoring key {} with unsupported type", key);
//             }
//         }
//     }

//     status_report
// }

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
    //let messages = get_list_data(&mut conn, id, "messages").await;
    //let errors = get_list_data(&mut conn, id, "errors").await;

    let mut status_report = HashMap::new();

    // Combine worker info
    for (key, data) in worker_info {
        status_report.insert(key, serde_json::json!(data));
    }

    // Combine messages
    // for (key, data) in messages {
    //     status_report.insert(
    //         key,
    //         serde_json::json!({
    //             "messages": data
    //         }),
    //     );
    // }

    // Combine errors
    // for (key, data) in errors {
    //     status_report.insert(
    //         key,
    //         serde_json::json!({
    //             "errors": data
    //         }),
    //     );
    // }

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

// async fn get_list_data(
//     conn: &mut redis::aio::MultiplexedConnection,
//     id: Uuid,
//     list_type: &str, // Should be "messages" or "errors"
// ) -> HashMap<String, Vec<serde_json::Value>> {
//     let pattern = format!("status:{}:*:*:*:{}", id, list_type);
//     let keys: Vec<String> = conn.keys(&pattern).await.unwrap();

//     let mut list_data = HashMap::new();

//     for key in keys {
//         let list_items: Vec<String> = conn.lrange(&key, 0, -1).await.unwrap_or_default();
//         let deserialized_list: Vec<serde_json::Value> = list_items
//             .into_iter()
//             .map(|item| {
//                 serde_json::from_str(&item)
//                     .unwrap_or_else(|_| serde_json::json!({"error": "Failed to deserialize JSON"}))
//             })
//             .collect();

//         list_data.insert(key, deserialized_list);
//     }

//     list_data
// }
