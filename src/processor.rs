use redis::AsyncCommands;

use crate::{
    models::{ApplicationState, StatusMessage},
    redis::{check_all_workers_completed, check_worker_counts, update_run_status},
};

pub async fn process_status_message(
    state: &ApplicationState,
    status_message: StatusMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await?;

    let base_key = format!(
        "status:{}:{}:worker:{}",
        status_message.id, status_message.client_id, status_message.worker_id
    );

    let info_key = format!("{}:info", base_key);

    match status_message.service.as_str() {
        "worker" => match status_message.action.as_str() {
            "started" => {
                conn.hset_multiple(
                    &info_key,
                    &[
                        ("vendor", status_message.vendor),
                        ("started", status_message.timestamp),
                        ("ended", "".to_string()),
                        ("completed", "false".to_string()),
                        ("worker_count", "0".to_string()),
                        ("worker_error_count", "0".to_string()),
                        ("feeder_count", "0".to_string()),
                        ("feeder_error_count", "0".to_string()),
                        ("warehouser_count", "0".to_string()),
                        ("warehouser_error_count", "0".to_string()),
                    ],
                )
                .await?;

                update_run_status(
                    &mut conn,
                    status_message.id,
                    status_message.client_id,
                    "started",
                )
                .await?;
            }
            "message_created" => {
                conn.hincr(&info_key, "worker_count", 1).await?;
            }
            "error" => {
                conn.hincr(&info_key, "worker_error_count", 1).await?;
            }
            "done" => {
                conn.hset(&info_key, "completed", "true").await?;
                conn.hset(&info_key, "ended", status_message.timestamp)
                    .await?;
            }
            _ => println!("Unknown worker action: {}", status_message.action),
        },
        "feeder" => match status_message.action.as_str() {
            "rules_applied" => {
                conn.hincr(&info_key, "feeder_count", 1).await?;
            }
            "error" => {
                conn.hincr(&info_key, "feeder_error_count", 1).await?;
            }
            _ => println!("Unknown feeder action: {}", status_message.action),
        },
        "warehouser" => match status_message.action.as_str() {
            "message_stored" => {
                conn.hincr(&info_key, "warehouser_count", 1).await?;
            }
            "error" => {
                conn.hincr(&info_key, "warehouser_error_count", 1).await?;
            }
            _ => println!("Unknown warehouser action: {}", status_message.action),
        },
        _ => println!("Unknown service type: {}", status_message.service),
    }

    let all_workers_done = check_all_workers_completed(&state, status_message.id).await?;
    let serivce_count_equal =
        check_worker_counts(&state, status_message.id, status_message.client_id).await?;

    if all_workers_done && serivce_count_equal {
        //println!("Workers, Feeder, & Warehouser have all processed the same number of messages");
        update_run_status(
            &mut conn,
            status_message.id,
            status_message.client_id,
            "completed",
        )
        .await?;
    } else {
        //println!("Not all services are done processing the run, yet");
        update_run_status(
            &mut conn,
            status_message.id,
            status_message.client_id,
            "pending",
        )
        .await?;
    };

    Ok(())
}
