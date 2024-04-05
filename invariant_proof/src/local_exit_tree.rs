use crate::utils::keccak256_merge;

#[derive(Clone, Debug)]
pub struct LocalExitTree {
    num_entries: u32,
    frontier: [[u8; 32]; 32],
}

impl LocalExitTree {
    pub fn new() -> Self {
        LocalExitTree {
            num_entries: 0,
            frontier: [[0u8; 32]; 32],
        }
    }

    pub fn add_leaf(&mut self, leaf: [u8; 32]) {
        // the index to which the new entry will be inserted
        let frontier_insertion_index: usize = {
            let new_num_entries = self.num_entries + 1;

            new_num_entries
                .trailing_zeros()
                .try_into()
                .expect("usize expected to be at least 32 bits")
        };

        // the new entry to be inserted in the frontier
        let new_frontier_entry = {
            let mut entry = leaf;
            for frontier_ele in &self.frontier[0..frontier_insertion_index] {
                entry = keccak256_merge(frontier_ele, &entry);
            }

            entry
        };

        // update tree
        self.frontier[frontier_insertion_index] = new_frontier_entry;
        self.num_entries += 1;
    }
}

impl Default for LocalExitTree {
    fn default() -> Self {
        Self::new()
    }
}
