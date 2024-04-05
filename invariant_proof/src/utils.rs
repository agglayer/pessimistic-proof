use tiny_keccak::{Hasher, Keccak};

pub fn keccak256_merge(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut keccak256 = Keccak::v256();
    keccak256.update(left);
    keccak256.update(right);

    let mut output = [0u8; 32];
    keccak256.finalize(&mut output);
    output
}
