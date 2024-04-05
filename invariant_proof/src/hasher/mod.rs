pub mod keccak;

pub trait Hasher {
    type Digest;

    fn merge(left: &Self::Digest, right: &Self::Digest) -> Self::Digest;
}
