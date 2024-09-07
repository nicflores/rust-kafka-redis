// pub struct State {
//     pub worker_count: usize,
//     pub done_count: usize,
//     pub feeder_count: usize,
//     pub warehouser_count: usize,
// }

// pub struct ApplicationState {
//     pub worker_counts: HashMap<(Uuid, u64), usize>,
//     pub done_counts: HashMap<(Uuid, u64), usize>,
//     pub feeder_counts: HashMap<(Uuid, u64), usize>,
//     pub warehouser_counts: HashMap<(Uuid, u64), usize>,
// }

// impl ApplicationState {
//     pub fn new() -> Self {
//         Self {
//             worker_counts: HashMap::new(),
//             done_counts: HashMap::new(),
//             feeder_counts: HashMap::new(),
//             warehouser_counts: HashMap::new(),
//         }
//     }

//     pub fn update_worker_started(&mut self, id: Uuid, client_id: u64) {
//         let key = (id, client_id);
//         *self.worker_counts.entry(key).or_insert(0) += 1;
//     }

//     pub fn update_done(&mut self, id: Uuid, client_id: u64, count: usize) {
//         let key = (id, client_id);
//         *self.done_counts.entry(key).or_insert(0) += count;
//     }

//     pub fn update_feeder(&mut self, id: Uuid, client_id: u64) {
//         let key = (id, client_id);
//         *self.feeder_counts.entry(key).or_insert(0) += 1;
//     }

//     pub fn update_warehouser(&mut self, id: Uuid, client_id: u64) {
//         let key = (id, client_id);
//         *self.warehouser_counts.entry(key).or_insert(0) += 1;
//     }

//     pub fn is_complete(&self, id: Uuid, client_id: u64) -> bool {
//         // Implement logic to determine if processing is complete
//         let key = (id, client_id);
//         if let Some(done_count) = self.done_counts.get(&key) {
//             if let Some(feeder_count) = self.feeder_counts.get(&key) {
//                 if let Some(warehouser_count) = self.warehouser_counts.get(&key) {
//                     return *done_count == *feeder_count && *done_count == *warehouser_count;
//                 }
//             }
//         }
//         false
//     }

//     pub fn process_complete(&mut self, id: Uuid, client_id: u64) {
//         // Handle the completion logic, like cleanup
//         let key = (id, client_id);
//         self.worker_counts.remove(&key);
//         self.done_counts.remove(&key);
//         self.feeder_counts.remove(&key);
//         self.warehouser_counts.remove(&key);
//     }

//     pub fn to_json(&self) -> Value {
//         let worker_counts: HashMap<String, usize> = self
//             .worker_counts
//             .iter()
//             .map(|(&(id, client_id), &count)| (format!("{},{}", id, client_id), count))
//             .collect();

//         let done_counts: HashMap<String, usize> = self
//             .done_counts
//             .iter()
//             .map(|(&(id, client_id), &count)| (format!("{},{}", id, client_id), count))
//             .collect();

//         let feeder_counts: HashMap<String, usize> = self
//             .feeder_counts
//             .iter()
//             .map(|(&(id, client_id), &count)| (format!("{},{}", id, client_id), count))
//             .collect();

//         let warehouser_counts: HashMap<String, usize> = self
//             .warehouser_counts
//             .iter()
//             .map(|(&(id, client_id), &count)| (format!("{},{}", id, client_id), count))
//             .collect();

//         json!({
//             "worker_counts": worker_counts,
//             "done_counts": done_counts,
//             "feeder_counts": feeder_counts,
//             "warehouser_counts": warehouser_counts,
//         })
//     }
// }
