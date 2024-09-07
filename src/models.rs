use redis::Client as RedisClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StatusMessage {
    pub service: String,
    pub worker_id: Uuid,
    pub vendor: String,
    pub action: String,
    pub id: Uuid,
    pub timestamp: String,
    pub client_id: u64,
    pub payload: Value,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ErrorDetail {
    pub timestamp: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MessageDetail {
    pub timestamp: String,
    pub payload: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WorkerStatus {
    pub completed: bool,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WorkerInfo {
    pub vendor: String,
    pub started: String,
    pub status: WorkerStatus,
    pub errors: Vec<ErrorDetail>,
    pub messages: Vec<MessageDetail>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FeederWarehouserInfo {
    pub message_count: usize,
    pub messages: Vec<MessageDetail>,
    pub errors: Vec<ErrorDetail>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ClientState {
    pub workers: HashMap<Uuid, WorkerInfo>,
    pub feeder: FeederWarehouserInfo,
    pub warehouser: FeederWarehouserInfo,
}

pub struct ApplicationState {
    pub redis_client: RedisClient,
}

impl ApplicationState {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    // pub async fn update_client_state(
    //     &self,
    //     client_id: &str,
    //     client_state: &ClientState,
    // ) -> RedisResult<()> {
    //     let mut conn = self.state.get_multiplexed_async_connection().await?;
    //     let serialized = serde_json::to_string(client_state).map_err(|e| {
    //         RedisError::from((
    //             redis::ErrorKind::ParseError,
    //             "Failed to serialize client state",
    //             e.to_string(),
    //         ))
    //     })?;
    //     conn.set(client_id, serialized).await?;
    //     Ok(())
    // }

    // pub async fn get_client_state(&self, client_id: &str) -> RedisResult<Option<ClientState>> {
    //     let mut conn = self.state.get_multiplexed_async_connection().await?;
    //     let serialized: Option<String> = conn.get(client_id).await?;
    //     match serialized {
    //         Some(s) => serde_json::from_str(&s)
    //             .map_err(|e| {
    //                 RedisError::from((
    //                     redis::ErrorKind::ParseError,
    //                     "Failed to deserialize client state",
    //                     e.to_string(),
    //                 ))
    //             })
    //             .map(Some),
    //         None => Ok(None),
    //     }
    // }

    // pub async fn get_all_client_states(&self) -> RedisResult<HashMap<String, ClientState>> {
    //     let mut conn = self.state.get_multiplexed_async_connection().await?;
    //     let keys: Vec<String> = conn.keys("*").await?;
    //     let mut all_states = HashMap::new();
    //     for key in keys {
    //         if let Ok(Some(state)) = self.get_client_state(&key).await {
    //             all_states.insert(key, state);
    //         }
    //     }
    //     Ok(all_states)
    // }
}
