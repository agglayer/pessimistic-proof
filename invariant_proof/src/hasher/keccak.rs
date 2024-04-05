use super::Hasher;
use tiny_keccak::{Hasher as _, Keccak};

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

pub struct Keccak256Hasher;

impl Hasher for Keccak256Hasher {
    type Digest = [u8; 32];

    fn merge(left: &[u8; 32], right: &[u8; 32]) -> Self::Digest {
        let mut keccak256 = Keccak::v256();
        keccak256.update(left);
        keccak256.update(right);

        let mut output = [0u8; 32];
        keccak256.finalize(&mut output);
        output
    }
}
