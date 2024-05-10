#![no_main]

use poly_pessimistic_proof::{batch::Batch, generate_leaf_proof};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let batch = sp1_zkvm::io::read::<Batch>();

    let (new_local_exit_root, _aggregate) = generate_leaf_proof(batch).unwrap();

    sp1_zkvm::io::commit(&new_local_exit_root);
}
