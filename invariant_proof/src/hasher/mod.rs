mod keccak;
pub use keccak::{keccak256, Keccak256Hasher};

pub trait Hasher {
    type Digest;

    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}
