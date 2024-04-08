use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub mod hasher;
use hasher::Hasher;

pub mod withdrawal;

#[cfg(test)]
mod tests;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalExitTree<Digest, const TREE_DEPTH: usize = 32>
where
    Digest: Serialize + for<'a> Deserialize<'a>,
{
    leaf_count: u32,
    #[serde_as(as = "[_; TREE_DEPTH]")]
    frontier: [Digest; TREE_DEPTH],
}

impl<Digest, const TREE_DEPTH: usize> LocalExitTree<Digest, TREE_DEPTH>
where
    Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    pub fn new() -> Self {
        LocalExitTree {
            leaf_count: 0,
            frontier: [Digest::default(); TREE_DEPTH],
        }
    }

    pub fn from_leaves<H>(leaves: impl Iterator<Item = Digest>) -> Self
    where
        H: Hasher<Digest = Digest>,
    {
        let mut tree = Self::new();

        for leaf in leaves {
            tree.add_leaf::<H>(leaf);
        }

        tree
    }

    pub fn add_leaf<H>(&mut self, leaf: H::Digest)
    where
        H: Hasher<Digest = Digest>,
    {
        // the index at which the new entry will be inserted
        let frontier_insertion_index: usize = {
            let leaf_count_after_insertion = self.leaf_count + 1;

            leaf_count_after_insertion
                .trailing_zeros()
                .try_into()
                .expect("usize expected to be at least 32 bits")
        };

        // the new entry to be inserted in the frontier
        let new_frontier_entry = {
            let mut entry = leaf;
            for frontier_ele in &self.frontier[0..frontier_insertion_index] {
                entry = H::merge(frontier_ele, &entry);
            }

            entry
        };

        // update tree
        self.frontier[frontier_insertion_index] = new_frontier_entry;
        self.leaf_count += 1;
    }

    pub fn get_root<H>(&self) -> Digest
    where
        H: Hasher<Digest = Digest>,
    {
        let mut root = Digest::default();
        let mut empty_hash_at_height = Digest::default();

        for height in 0..TREE_DEPTH {
            if get_bit_at(self.leaf_count, height) == 1 {
                root = H::merge(&self.frontier[height], &root);
            } else {
                root = H::merge(&root, &empty_hash_at_height);
            }

            empty_hash_at_height = H::merge(&empty_hash_at_height, &empty_hash_at_height);
        }

        root
    }
}

impl<Digest, const TREE_DEPTH: usize> Default for LocalExitTree<Digest, TREE_DEPTH>
where
    Digest: Copy + Default + Serialize + for<'a> Deserialize<'a>,
{
    fn default() -> Self {
        Self::new()
    }
}

fn get_bit_at(target: u32, bit_idx: usize) -> u32 {
    (target >> bit_idx) & 1
}
