use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const JSON_FILE_PATH: &str = "tests/data/bridge_events_10k.json";

#[test]
fn test_local_exit_root() {
    let bridge_events: Vec<BridgeEvent> = {
        let json_file = File::open(JSON_FILE_PATH).unwrap();
        let reader = BufReader::new(json_file);

        serde_json::from_reader(reader).unwrap()
    };
}

#[derive(Serialize, Deserialize)]
struct BridgeEvent {
    removed: bool,
    block_number: u64,
    transaction_index: u64,
    log_index: u64,
    transaction_hash: String,
    event_type: u8,
    event_data: HashMap<String, Value>
}
