mod keccak;
pub use keccak::Keccak256;

pub trait Hasher {
    type Digest;

    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}
