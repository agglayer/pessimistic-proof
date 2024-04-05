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
