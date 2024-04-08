use num_bigint::BigInt;
use reth_primitives::{revm_primitives::bitvec::view::BitViewSized, Address};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use super::hasher::keccak::keccak256;

/// Represents a token withdrawal from the network.
#[derive(Serialize, Deserialize)]
pub struct Withdrawal {
    pub leaf_type: u8,

    pub orig_network: u32,
    pub orig_address: Address,

    pub dest_network: u32,
    pub dest_address: Address,

    pub amount: BigInt,

    pub metadata: Vec<u8>,
}

impl Withdrawal {
    /// Creates a new [`Withdrawal`].
    pub fn new(
        leaf_type: u8,
        orig_network: u32,
        orig_address: Address,
        dest_network: u32,
        dest_address: Address,
        amount: BigInt,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            leaf_type,
            orig_network,
            orig_address,
            dest_network,
            dest_address,
            amount,
            metadata,
        }
    }

    /// Hashes the [`Withdrawal`] to be inserted in a [`crate::local_exit_tree::LocalExitTree`].
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Keccak::v256();

        hasher.update(self.leaf_type.as_raw_slice());
        hasher.update(&u32::to_be_bytes(self.orig_network));
        hasher.update(self.orig_address.as_slice());
        hasher.update(&u32::to_be_bytes(self.dest_network));
        hasher.update(self.dest_address.as_slice());
        hasher.update(&self.amount_as_bytes());
        hasher.update(&keccak256(&self.metadata));

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }

    /// Prepares the `amount` field for hashing
    fn amount_as_bytes(&self) -> [u8; 32] {
        // FIXME: Ideally, we'd avoid using the heap for this calculation
        let mut amount_bytes = self.amount.to_signed_bytes_be();
        let padding_length = 32 - amount_bytes.len();

        let mut output = Vec::with_capacity(32);
        output.resize(padding_length, 0_u8);
        output.append(&mut amount_bytes);

        output.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_exit_tree::{
        hasher::keccak::{Keccak256Hasher, KeccakDigest},
        LocalExitTree,
    };

    #[test]
    fn test_deposit_hash() {
        let mut deposit = Withdrawal {
            leaf_type: 0,
            orig_network: 0,
            orig_address: Address::default(),
            dest_network: 1,
            dest_address: Address::default(),
            amount: BigInt::default(),
            metadata: vec![],
        };

        let amount_bytes = hex::decode("8ac7230489e80000").unwrap_or_default();
        deposit.amount = BigInt::from_signed_bytes_be(amount_bytes.as_slice());

        let dest_addr = hex::decode("c949254d682d8c9ad5682521675b8f43b102aec4").unwrap_or_default();
        deposit.dest_address.copy_from_slice(&dest_addr);

        let leaf_hash = deposit.hash();
        assert_eq!(
            "22ed288677b4c2afd83a6d7d55f7df7f4eaaf60f7310210c030fd27adacbc5e0",
            hex::encode(leaf_hash)
        );

        let mut dm = LocalExitTree::<KeccakDigest, Keccak256Hasher>::new();
        dm.add_leaf(leaf_hash);
        let dm_root = dm.get_root();
        assert_eq!(
            "5ba002329b53c11a2f1dfe90b11e031771842056cf2125b43da8103c199dcd7f",
            hex::encode(dm_root)
        );
    }
}
