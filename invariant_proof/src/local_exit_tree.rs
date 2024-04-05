use crate::hasher::Hasher;

pub const TREE_DEPTH: usize = 32;

#[derive(Clone, Debug)]
pub struct LocalExitTree<H: Hasher> {
    leaf_count: u32,
    frontier: [H::Digest; TREE_DEPTH],
}

impl<H> LocalExitTree<H>
where
    H: Hasher,
    H::Digest: Copy + Default,
{
    pub fn new() -> Self {
        LocalExitTree {
            leaf_count: 0,
            frontier: [H::Digest::default(); TREE_DEPTH],
        }
    }

    pub fn add_leaf(&mut self, leaf: H::Digest) {
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

    pub fn get_root(&self) -> H::Digest {
        let mut root = H::Digest::default();
        let mut empty_hash_at_height = H::Digest::default();

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

impl<H> Default for LocalExitTree<H>
where
    H: Hasher,
    H::Digest: Copy + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

fn get_bit_at(target: u32, bit_idx: usize) -> u32 {
    (target >> bit_idx) & 1
}
