#![no_main]

use poly_invariant_proof::{
    hasher::keccak::KeccakDigest,
    leaf_proof,
    local_exit_tree::{withdrawal::Withdrawal, LocalExitTree},
};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let local_exit_tree = sp1_zkvm::io::read::<LocalExitTree<KeccakDigest>>();
    let local_exit_root = sp1_zkvm::io::read::<KeccakDigest>();
    let withdrawals = sp1_zkvm::io::read::<Vec<Withdrawal>>();

    let (new_local_exit_root, aggregate_deposits) = leaf_proof(local_exit_tree, local_exit_root, withdrawals).unwrap();

    sp1_zkvm::io::commit(&new_local_exit_root);
    sp1_zkvm::io::commit(&aggregate_deposits.hash());
}
