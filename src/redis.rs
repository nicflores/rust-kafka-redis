use redis::{pipe, AsyncCommands};
use uuid::Uuid;

use crate::models::ApplicationState;

pub async fn update_run_status(
    conn: &mut redis::aio::MultiplexedConnection,
    run_id: Uuid,
    client_id: u64,
    status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Define the Redis key for the final run status
    let final_run_status = format!("status:{}:{}:info", run_id, client_id);

    // Set the "run_status" field to the provided status
    conn.hset(&final_run_status, "run_status", status).await?;
    println!("Set 'run_status' to '{}' for {}", status, final_run_status);
    Ok(())
}

pub async fn check_worker_counts(
    state: &ApplicationState,
    id: Uuid,
    client_id: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    // status:585e5d0c-a45a-4b2e-ba96-d2ce1b415665:1234:worker:f21a3159-f0b6-410d-b69b-136ddf1c085d:info
    let pattern = format!("status:{}:{}:worker:*:info", id, client_id);

    let keys: Vec<String> = conn.keys(&pattern).await.unwrap(); // Retrieve all keys matching the pattern

    if keys.is_empty() {
        return Ok(false);
    }

    let mut pipeline = pipe(); // Create a new pipeline

    // Queue HGET commands in the pipeline for count field
    for key in &keys {
        pipeline.cmd("HGET").arg(key).arg("worker_count");
        pipeline.cmd("HGET").arg(key).arg("feeder_count");
        pipeline.cmd("HGET").arg(key).arg("warehouser_count");
    }

    // Execute the pipeline and retrieve the results
    let results: Vec<Option<String>> = pipeline.query_async(&mut conn).await?;

    // Convert the results to usize and check if they are equal
    for chunk in results.chunks(3) {
        if chunk.len() == 3 {
            let worker_count = chunk[0]
                .as_ref()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let feeder_count = chunk[1]
                .as_ref()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let warehouser_count = chunk[2]
                .as_ref()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            // Check if the counts are equal for the current worker
            if worker_count != feeder_count || feeder_count != warehouser_count {
                return Ok(false); // If any mismatch is found, return false immediately
            }
        }
    }

    Ok(true)
}

pub async fn check_all_workers_completed(
    state: &ApplicationState,
    id: Uuid,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    let pattern = format!("status:{}:*:worker:*:info", id);

    let keys: Vec<String> = conn.keys(&pattern).await.unwrap();
    let mut pipeline = pipe();

    // Queue HGET commands in the pipeline for count field
    for key in &keys {
        pipeline.cmd("HGET").arg(key).arg("completed");
    }

    // Execute the pipeline and retrieve the results
    let counts: Vec<Option<String>> = pipeline.query_async(&mut conn).await?;

    // Sum the results to get the total message count
    let all_true = counts.into_iter().all(|x| x == Some("true".to_string()));

    Ok(all_true)
}
