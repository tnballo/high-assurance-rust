use core::ops::Sub;

use super::node_dispatch::SmallNode;

use smallnum::SmallUnsigned;
use tinyvec::ArrayVec;

/*
Note:

Structures in this file generic for `U` in a *subset* of the set `(u8, u16, u32, u64, u128)`.
All members in subset are <= host pointer width in size.
If caller obeys contract, `U` will be smallest unsigned capable of representing `arena::Arena`'s
const `N` (e.g. static capacity).
*/

// Tree Node -----------------------------------------------------------------------------------------------------------

/// Binary tree node, meta programmable for low memory footprint.
/// Users of it's APIs only need to declare `U` type or trait bounds at construction.
/// All APIs take/return `usize` and normalize to `U` internally.
#[derive(Clone, Debug, Default)]
pub struct Node<K, V, U> {
    key: K,
    val: V,
    left_idx: Option<U>,
    right_idx: Option<U>,

    #[cfg(feature = "fast_rebalance")]
    subtree_size: U,
}

impl<K, V, U: SmallUnsigned> Node<K, V, U> {
    /// Constructor.
    pub fn new(key: K, val: V) -> Self {
        Node {
            key,
            val,
            left_idx: None,
            right_idx: None,

            #[cfg(feature = "fast_rebalance")]
            subtree_size: U::checked_from(1),
        }
    }
}

impl<K: Default, V: Default, U: SmallUnsigned + Copy> SmallNode<K, V> for Node<K, V, U> {
    fn key(&self) -> &K {
        &self.key
    }

    fn set_key(&mut self, key: K) {
        self.key = key;
    }

    fn take_key(&mut self) -> K {
        core::mem::take(&mut self.key)
    }

    fn val(&self) -> &V {
        &self.val
    }

    fn get_mut(&mut self) -> (&K, &mut V) {
        (&self.key, &mut self.val)
    }

    fn take_val(&mut self) -> V {
        core::mem::take(&mut self.val)
    }

    fn set_val(&mut self, val: V) {
        self.val = val;
    }

    fn left_idx(&self) -> Option<usize> {
        self.left_idx.map(|i| i.usize())
    }

    fn set_left_idx(&mut self, opt_idx: Option<usize>) {
        match opt_idx {
            Some(idx) => self.left_idx = Some(U::checked_from(idx)),
            None => self.left_idx = None,
        }
    }

    fn right_idx(&self) -> Option<usize> {
        self.right_idx.map(|i| i.usize())
    }

    fn set_right_idx(&mut self, opt_idx: Option<usize>) {
        match opt_idx {
            Some(idx) => self.right_idx = Some(U::checked_from(idx)),
            None => self.right_idx = None,
        }
    }

    #[cfg(feature = "fast_rebalance")]
    fn subtree_size(&self) -> usize {
        self.subtree_size.usize()
    }

    #[cfg(feature = "fast_rebalance")]
    fn set_subtree_size(&mut self, size: usize) {
        self.subtree_size = U::checked_from(size);
    }
}

// Retrieval Helper ----------------------------------------------------------------------------------------------------

/// Helper for node retrieval, usage eliminates the need a store parent pointer in each node.
/// Users of it's APIs only need to declare `U` type or trait bounds at construction.
/// All APIs take/return `usize` and normalize to `U` internally.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NodeGetHelper<U> {
    node_idx: Option<U>,
    parent_idx: Option<U>,
    is_right_child: bool,
}

impl<U: SmallUnsigned + Copy> NodeGetHelper<U> {
    /// Constructor.
    pub fn new(node_idx: Option<usize>, parent_idx: Option<usize>, is_right_child: bool) -> Self {
        NodeGetHelper {
            node_idx: node_idx.map(|i| U::checked_from(i)),
            parent_idx: parent_idx.map(|i| U::checked_from(i)),
            is_right_child,
        }
    }

    /// Get node index as `usize`
    pub fn node_idx(&self) -> Option<usize> {
        self.node_idx.map(|i| i.usize())
    }

    /// Get parent index as `usize`
    pub fn parent_idx(&self) -> Option<usize> {
        self.parent_idx.map(|i| i.usize())
    }

    // Tell if right or left child
    pub fn is_right_child(&self) -> bool {
        self.is_right_child
    }
}

// Tree Rebuild Helper -------------------------------------------------------------------------------------------------

/// Helper for in-place iterative rebuild.
/// Users of it's APIs only need to declare `U` type or trait bounds at construction.
/// All APIs take/return `usize` and normalize to `U` internally.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NodeRebuildHelper<U> {
    pub low_idx: U,
    pub high_idx: U,
    pub mid_idx: U,
}

impl<U: SmallUnsigned + Ord + Sub + Copy> NodeRebuildHelper<U> {
    /// Constructor.
    pub fn new(low_idx: usize, high_idx: usize) -> Self {
        debug_assert!(
            high_idx >= low_idx,
            "Node rebuild helper low/high index reversed!"
        );

        NodeRebuildHelper {
            low_idx: U::checked_from(low_idx),
            high_idx: U::checked_from(high_idx),
            mid_idx: U::checked_from(low_idx + ((high_idx - low_idx) / 2)),
        }
    }
}

// Swap History Cache --------------------------------------------------------------------------------------------------

/// A helper "cache" for swap operation history.
/// If every index swap is logged, tracks mapping of original to current indexes.
/// Users of it's APIs only need to declare `U` type or trait bounds at construction.
/// All APIs take/return `usize` and normalize to `U` internally.
#[derive(Debug, Default)]
pub struct NodeSwapHistHelper<U: Default, const N: usize> {
    /// Map `original_idx` -> `current_idx`
    history: ArrayVec<[(U, U); N]>,
}

impl<U: Ord + Default + Copy + SmallUnsigned, const N: usize> NodeSwapHistHelper<U, N> {
    /// Constructor.
    pub fn new() -> Self {
        NodeSwapHistHelper {
            history: ArrayVec::<[(U, U); N]>::default(),
        }
    }

    /// Log the swap of elements at two indexes.
    /// Every swap performed must be logged with this method for the cache to remain accurate.
    pub fn add(&mut self, pos_1: usize, pos_2: usize) {
        debug_assert_ne!(pos_1, pos_2);

        let mut known_pos_1 = false;
        let mut known_pos_2 = false;

        let pos_1 = U::checked_from(pos_1);
        let pos_2 = U::checked_from(pos_2);

        // Update existing
        for (_, curr_idx) in self.history.iter_mut() {
            if *curr_idx == pos_1 {
                *curr_idx = pos_2;
                known_pos_1 = true;
            } else if *curr_idx == pos_2 {
                *curr_idx = pos_1;
                known_pos_2 = true;
            }
        }

        // Add new mapping
        if !known_pos_1 {
            self.history.push((pos_1, pos_2));
        }

        // Add new mapping
        if !known_pos_2 {
            self.history.push((pos_2, pos_1));
        }
    }

    /// Retrieve the current value of an original index from the map.
    pub fn curr_idx(&self, orig_pos: usize) -> usize {
        debug_assert!(
            self.history
                .iter()
                .filter(|(k, _)| (*k).usize() == orig_pos)
                .count()
                <= 1
        );

        match self
            .history
            .iter()
            .filter(|(k, _)| (*k).usize() == orig_pos)
            .map(|(_, curr)| *curr)
            .next()
        {
            Some(curr_idx) => curr_idx.usize(),
            None => orig_pos,
        }
    }
}

// Test ----------------------------------------------------------------------------------------------------------------

// Note: low_mem_insert feature doesn't affect node size, only arena size.
#[cfg(not(feature = "low_mem_insert"))]
#[cfg(test)]
mod tests {
    use super::Node;
    use smallnum::small_unsigned;
    use std::mem::size_of;

    #[test]
    fn test_node_sizing() {
        // No features
        #[cfg(target_pointer_width = "64")]
        #[cfg(not(feature = "fast_rebalance"))]
        {
            assert_eq!(size_of::<Node<u32, u32, small_unsigned!(1024)>>(), 16);
        }

        // fast_rebalance only
        #[cfg(target_pointer_width = "64")]
        #[cfg(feature = "fast_rebalance")]
        {
            assert_eq!(size_of::<Node<u32, u32, small_unsigned!(1024)>>(), 20);
        }
    }
}
