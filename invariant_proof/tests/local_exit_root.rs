use std::{fs::File, io::BufReader};

use base64::{engine::general_purpose::STANDARD, Engine};
use poly_invariant_proof::{
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    TokenInfo, Withdrawal,
};
use reth_primitives::{Address, U256};
use serde::{Deserialize, Deserializer};
use serde_json::Number;

const JSON_FILE_PATH: &str = "tests/data/bridge_events_10k.json";

#[test]
fn test_local_exit_root() {
    let mut local_exit_tree: LocalExitTree<Keccak256Hasher> = LocalExitTree::new();

    let bridge_events: Vec<BridgeEvent> = read_sorted_bridge_events();

    let mut deposit_count: u32 = 0;
    for event in bridge_events {
        match event.event_data {
            EventData::UpdateL1InfoTree {
                mainnet_exit_root,
                rollup_exit_root: _,
            } => {
                let computed_root = local_exit_tree.get_root();

                assert_eq!(computed_root, mainnet_exit_root);
            }
            EventData::Deposit(deposit_event_data) => {
                assert_eq!(deposit_event_data.deposit_count, deposit_count);
                deposit_count += 1;

                let withdrawal: Withdrawal = deposit_event_data.into();
                local_exit_tree.add_leaf(withdrawal.hash());
            }
            EventData::Claim(_) => {
                // do nothing
            }
        }
    }
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

#[allow(unused)]
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

#[allow(unused)]
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
    #[serde(deserialize_with = "u256_from_number")]
    amount: U256,
    metadata: String,
    deposit_count: u32,
}

impl From<DepositEventData> for Withdrawal {
    fn from(deposit_event_data: DepositEventData) -> Self {
        Self {
            leaf_type: deposit_event_data.leaf_type,
            token_info: TokenInfo {
                origin_network: deposit_event_data.origin_network.into(),
                origin_token_address: Address::parse_checksummed(
                    deposit_event_data.origin_address,
                    None,
                )
                .unwrap(),
            },
            dest_network: deposit_event_data.destination_network.into(),
            dest_address: Address::parse_checksummed(deposit_event_data.destination_address, None)
                .unwrap(),
            amount: deposit_event_data.amount,
            metadata: STANDARD.decode(deposit_event_data.metadata).unwrap(),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaimEventData {
    #[serde(deserialize_with = "u256_from_number")]
    #[serde(rename = "index")]
    global_index: U256,
    origin_network: u32,
    origin_address: String,
    destination_address: String,
    #[serde(deserialize_with = "u256_from_number")]
    amount: U256,
}

fn u256_from_number<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let n = Number::deserialize(deserializer)?;

    Ok(U256::from_str_radix(n.as_str(), 10).unwrap())
}
