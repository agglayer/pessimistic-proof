use std::{fs::File, io::BufReader};

use num_bigint::BigUint;
use num_traits::FromPrimitive;
use serde::{Deserialize, Deserializer};
use serde_json::Number;

const JSON_FILE_PATH: &str = "tests/data/bridge_events_10k.json";

#[test]
fn test_local_exit_root() {
    let bridge_events: Vec<BridgeEvent> = read_sorted_bridge_events();
}

/// Reads the bridge events from disk, and sorts by (block number, tx index, log index).
fn read_sorted_bridge_events() -> Vec<BridgeEvent> {
    let json_file = File::open(JSON_FILE_PATH).unwrap();
    let reader = BufReader::new(json_file);

    let mut bridge_events: Vec<BridgeEvent> = serde_json::from_reader(reader).unwrap();
    bridge_events.sort_unstable_by(|a, b| {
        use std::cmp::Ordering;
        match a.block_number.cmp(&b.block_number) {
            Ordering::Equal => match a.transaction_index.cmp(&b.transaction_index) {
                Ordering::Equal => a.log_index.cmp(&b.log_index),
                not_eq => not_eq,
            },
            not_eq => not_eq,
        }
    });

    bridge_events
}

#[derive(Debug, Deserialize)]
struct BridgeEvent {
    removed: bool,
    block_number: u64,
    transaction_index: u64,
    log_index: u64,
    transaction_hash: String,
    event_type: u8,
    event_data: EventData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EventData {
    // Mainnet exit root update event
    #[serde(rename_all = "camelCase")]
    UpdateL1InfoTree {
        mainnet_exit_root: [u8; 32],
        rollup_exit_root: [u8; 32],
    },
    // Deposit event
    Deposit(DepositEventData),
    Claim(ClaimEventData),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepositEventData {
    leaf_type: u8,
    origin_network: u32,
    origin_address: String,
    destination_network: u32,
    destination_address: String,
    #[serde(deserialize_with = "biguint_from_number")]
    amount: BigUint,
    metadata: String,
    deposit_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaimEventData {
    #[serde(deserialize_with = "biguint_from_number")]
    #[serde(rename = "index")]
    global_index: BigUint,
    origin_network: u32,
    origin_address: String,
    destination_address: String,
    #[serde(deserialize_with = "biguint_from_number")]
    amount: BigUint,
}

// hack to properly deserialize BigUints
fn biguint_from_number<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let n = Number::deserialize(deserializer)?;
    if let Some(u) = n.as_u64() {
        return Ok(BigUint::from(u));
    }
    if let Some(f) = n.as_f64() {
        return BigUint::from_f64(f).ok_or_else(|| {
            <D::Error as serde::de::Error>::invalid_value(
                serde::de::Unexpected::Float(f),
                &"a finite value",
            )
        });
    }

    panic!("biguint_from_number needs to be fixed")
}
