use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut};

use super::node::{Node, NodeGetHelper, NodeSwapHistHelper};
use super::node_dispatch::SmallNode;

use smallnum::SmallUnsigned;
use tinyvec::ArrayVec;

/*
Note:

Structures in this file generic for `U` in a *subset* of the set `(u8, u16, u32, u64, u128)`.
All members in subset are <= host pointer width in size.
If caller obeys contract, `U` will be smallest unsigned capable of representing const `N` (e.g. static capacity).
*/

/// An arena allocator, meta programmable for low memory footprint.
#[derive(Clone, Debug)]
pub struct Arena<K: Default, V: Default, U: Default, const N: usize> {
    vec: ArrayVec<[Option<Node<K, V, U>>; N]>,

    #[cfg(not(feature = "low_mem_insert"))]
    free_list: ArrayVec<[U; N]>,
}

impl<
        K: Default,
        V: Default,
        U: Default + Copy + SmallUnsigned + Ord + PartialEq + PartialOrd,
        const N: usize,
    > Arena<K, V, U, N>
{
    // TODO: is this function necessary?
    /// Const associated constructor for index scratch vector.
    pub fn new_idx_vec() -> ArrayVec<[U; N]> {
        ArrayVec::<[U; N]>::default()
    }

    /// Constructor.
    pub fn new() -> Self {
        let a = Arena {
            vec: ArrayVec::<[Option<Node<K, V, U>>; N]>::new(),

            #[cfg(not(feature = "low_mem_insert"))]
            free_list: ArrayVec::<[U; N]>::new(),
        };

        #[cfg(not(feature = "low_mem_insert"))]
        debug_assert_eq!(0, a.free_list.len());
        debug_assert_eq!(0, a.vec.len());

        #[cfg(not(feature = "low_mem_insert"))]
        debug_assert_eq!(N, a.free_list.capacity());
        debug_assert_eq!(N, a.vec.capacity());

        a
    }
    /// Returns an iterator over immutable arena elements.
    pub fn iter(&self) -> Iter<'_, Option<Node<K, V, U>>> {
        self.vec.iter()
    }

    /// Returns an iterator over arena elements that allows modifying each value.
    pub fn iter_mut(&mut self) -> IterMut<'_, Option<Node<K, V, U>>> {
        self.vec.iter_mut()
    }

    /// Total capacity, e.g. maximum number of items.
    pub fn capacity(&self) -> usize {
        N
    }

    /// Add node to area, growing if necessary, and return addition index.
    pub fn add(&mut self, key: K, val: V) -> usize {
        // O(1) find, constant time
        #[cfg(not(feature = "low_mem_insert"))]
        let opt_free_idx = self.free_list.pop();

        // O(n) find, linear search
        #[cfg(feature = "low_mem_insert")]
        let opt_free_idx = self
            .vec
            .iter()
            .position(|x| x.is_none())
            .map(|i| U::checked_from(i));

        let node = Node::new(key, val);
        match opt_free_idx {
            Some(free_idx) => {
                debug_assert!(
                    self.vec[free_idx.usize()].is_none(),
                    "Internal invariant failed: overwrite of allocated node!"
                );
                self.vec[free_idx.usize()] = Some(node);
                free_idx.usize()
            }
            None => {
                self.vec.push(Some(node));
                self.vec.len() - 1
            }
        }
    }

    /// Remove node at a given index from area, return it.
    pub fn remove(&mut self, idx: usize) -> Option<Node<K, V, U>> {
        debug_assert!(
            idx < self.vec.len(),
            "API misuse: requested removal past last index!"
        );

        if self.is_occupied(idx) {
            // Extract node
            let node = core::mem::replace(&mut self.vec[idx], None);

            // Append removed index to free list
            #[cfg(not(feature = "low_mem_insert"))]
            self.free_list.push(U::checked_from(idx));

            return node;
        }

        None
    }

    /// Remove node at a known-good index (simpler callsite and error handling) from area.
    /// This function can panic. If the index might be invalid, use `remove` instead.
    pub fn hard_remove(&mut self, idx: usize) -> Node<K, V, U> {
        match self.remove(idx) {
            Some(node) => node,
            None => {
                panic!("Internal invariant failed: attempted removal of node from invalid index.")
            }
        }
    }

    /// Sort the arena in caller-requested order and update all tree metadata accordingly
    /// `unwraps` will never panic if caller invariants upheld (checked via `debug_assert`)
    pub fn sort(
        &mut self,
        root_idx: usize,
        sort_metadata: ArrayVec<[NodeGetHelper<usize>; N]>, // `usize` here avoids `U` in tree iter signatures
    ) -> usize {
        debug_assert!(sort_metadata.iter().all(|ngh| ngh.node_idx().is_some()));

        let mut swap_history = NodeSwapHistHelper::<U, N>::new();

        // Sort as requested
        for (sorted_idx, ngh) in sort_metadata.iter().enumerate() {
            let curr_idx = swap_history.curr_idx(ngh.node_idx().unwrap());
            if curr_idx != sorted_idx {
                self.vec.swap(curr_idx, sorted_idx);
                swap_history.add(curr_idx, sorted_idx);

                // TODO: move this out of loop body, should do once at end of func with `swap_history`
                #[cfg(not(feature = "low_mem_insert"))]
                {
                    let old_free_idx = U::checked_from(sorted_idx);
                    let new_free_idx = U::checked_from(curr_idx);
                    self.free_list.iter_mut().for_each(|i| {
                        if *i == old_free_idx {
                            *i = new_free_idx;
                        }
                    });
                }
            }
        }

        // Update all parent-child relationships
        for ngh in sort_metadata {
            if let Some(parent_idx) = ngh.parent_idx() {
                let curr_parent_idx = swap_history.curr_idx(parent_idx);
                let curr_child_idx = swap_history.curr_idx(ngh.node_idx().unwrap());
                let parent_node = &mut self[curr_parent_idx];
                if ngh.is_right_child() {
                    parent_node.set_right_idx(Some(curr_child_idx));
                } else {
                    parent_node.set_left_idx(Some(curr_child_idx));
                }
            }
        }

        // Report new root
        swap_history.curr_idx(root_idx)
    }

    /// Returns the number of entries in the arena, some of which may be `None`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns true if the index is occupied, e.g. `Some(node)`.
    pub fn is_occupied(&self, idx: usize) -> bool {
        (idx < self.vec.len()) && (self.vec[idx].is_some())
    }

    /// Get the size of an individual arena node, in bytes.
    pub fn node_size(&self) -> usize {
        core::mem::size_of::<Node<K, V, U>>()
    }
}

// Convenience Traits --------------------------------------------------------------------------------------------------

/// Immutable indexing.
/// Indexed location MUST be occupied.
impl<K: Default, V: Default, U: Default, const N: usize> Index<usize> for Arena<K, V, U, N> {
    type Output = Node<K, V, U>;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.vec[index] {
            Some(node) => node,
            None => unreachable!(),
        }
    }
}

/// Mutable indexing
/// Indexed location MUST be occupied.
impl<K: Default, V: Default, U: Default, const N: usize> IndexMut<usize> for Arena<K, V, U, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.vec.index_mut(index) {
            Some(node) => node,
            None => unreachable!(),
        }
    }
}

impl<
        K: Ord + Default,
        V: Default,
        U: Default + Copy + SmallUnsigned + Ord + PartialEq + PartialOrd,
        const N: usize,
    > Default for Arena<K, V, U, N>
{
    fn default() -> Self {
        Self::new()
    }
}

/*
NOTE: This is draft code for upgrades when `feature(generic_const_exprs)` stabilizes.

// Wrapper Iterators ---------------------------------------------------------------------------------------------------

pub struct ArenaIter<'a, K: Default, V: Default, U, const N: usize> {
    arena_iter: core::slice::Iter<'a, Option<Node<K, V, U>>>,
}

impl<'a, K: Default, V: Default, U, const N: usize> ArenaIter<'a, K, V, U, N> {
    pub fn new(arena: &'a Arena<K, V, U, N>) -> Self {
        ArenaIter {
            arena_iter: arena.vec.iter(),
        }
    }
}

impl<'a, K: Default, V: Default, U: SmallUnsigned + Copy, const N: usize> Iterator for ArenaIter<'a, K, V, U, N> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.arena_iter.next() {
            Some(Some(node)) => Some((node.key(), node.val())),
            _ => None,
        }
    }
}

pub struct ArenaIterMut<'a, K: Default, V: Default, U, const N: usize> {
    arena_iter_mut: core::slice::IterMut<'a, Option<Node<K, V, U>>>,
}

impl<'a, K: Default, V: Default, U, const N: usize> ArenaIterMut<'a, K, V, U, N> {
    pub fn new(arena: &'a mut Arena<K, V, U, N>) -> Self {
        ArenaIterMut {
            arena_iter_mut: arena.vec.iter_mut(),
        }
    }
}

impl<'a, K: Default, V: Default, U: SmallUnsigned + Copy, const N: usize> Iterator for ArenaIterMut<'a, K, V, U, N> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.arena_iter_mut.next() {
            Some(Some(node)) => Some(node.get_mut()),
            _ => None,
        }
    }
}
*/

// Test ----------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Arena;
    use crate::tree::node::NodeGetHelper;
    use crate::tree::node_dispatch::SmallNode;
    use core::mem::size_of_val;
    use smallnum::small_unsigned;
    use tinyvec::array_vec;

    const CAPACITY: usize = 1024;

    #[test]
    fn test_add_and_remove() {
        let mut arena: Arena<isize, &str, small_unsigned!(CAPACITY), CAPACITY> = Arena::new();

        let n_1_idx = arena.add(1, "n/a");
        let n_2_idx = arena.add(2, "n/a");
        let n_3_idx = arena.add(3, "n/a");

        assert_eq!(n_1_idx, 0);
        assert_eq!(n_2_idx, 1);
        assert_eq!(n_3_idx, 2);

        let n_2_removed = arena.remove(n_2_idx).unwrap();
        assert_eq!(n_2_removed.key(), &2);
        assert!(arena.vec[1].is_none());

        let n_4_idx = arena.add(4, "n/a");
        assert_eq!(n_4_idx, 1);

        let n_5_idx = arena.add(5, "n/a");
        assert_eq!(n_5_idx, 3);
    }

    #[test]
    fn test_index_mut() {
        let mut arena: Arena<isize, &str, small_unsigned!(CAPACITY), CAPACITY> = Arena::new();
        let n_1_idx = arena.add(1, "n/a");
        assert_eq!(arena[n_1_idx].val(), &"n/a");
        let n_1_mut_ref = &mut arena[n_1_idx];
        n_1_mut_ref.set_val("This is a value. There are many like it but this one is mine.");
        assert_ne!(arena[n_1_idx].val(), &"n/a");
    }

    #[test]
    fn test_index_1() {
        let mut arena: Arena<u64, &str, small_unsigned!(CAPACITY), CAPACITY> = Arena::new();
        let n_1_idx = arena.add(0xD00DFEED_u64, "n/a");
        let n_1_ref = &arena[n_1_idx];
        assert_eq!(n_1_ref.key(), &0xD00DFEED_u64);
    }

    #[test]
    #[should_panic]
    fn test_index_2() {
        let mut arena: Arena<u64, &str, small_unsigned!(CAPACITY), CAPACITY> = Arena::new();
        arena.add(0xD00DFEED_u64, "n/a");
        let _ = &arena[1]; // OOB
    }

    #[test]
    fn test_capacity() {
        let arena = Arena::<i8, u128, small_unsigned!(CAPACITY), CAPACITY>::new();
        assert_eq!(arena.capacity(), CAPACITY);

        let arena = Arena::<i32, &str, small_unsigned!(1337), 1337>::new();
        assert_eq!(arena.capacity(), 1337);
    }

    #[test]
    fn test_sort() {
        let mut arena = Arena::<usize, &str, small_unsigned!(CAPACITY), CAPACITY>::new();

        // Simple 3-node tree:
        //
        //     2
        //     |
        // ---------
        // |       |
        // 1       3
        //
        arena.add(3, "n/a");
        let n_2_idx = arena.add(2, "n/a");
        arena.add(1, "n/a");

        let n_2 = &mut arena[n_2_idx];
        n_2.set_left_idx(Some(2));
        n_2.set_right_idx(Some(0));

        // Unsorted (insertion/"physical" order)
        assert_eq!(arena.vec[0].as_ref().unwrap().key(), &3);
        assert_eq!(arena.vec[1].as_ref().unwrap().key(), &2);
        assert_eq!(arena.vec[2].as_ref().unwrap().key(), &1);

        // Would be supplied for the above tree
        let sort_metadata = array_vec! { [NodeGetHelper<usize>; CAPACITY] =>
            NodeGetHelper::new(Some(2), Some(1), false),
            NodeGetHelper::new(Some(1), None, false),
            NodeGetHelper::new(Some(0), Some(1), false),
        };

        arena.sort(1, sort_metadata);

        // Sorted ("logical" order)
        assert_eq!(arena.vec[0].as_ref().unwrap().key(), &1);
        assert_eq!(arena.vec[1].as_ref().unwrap().key(), &2);
        assert_eq!(arena.vec[2].as_ref().unwrap().key(), &3);
    }

    #[test]
    fn test_node_packing() {
        const SMALL_CAPACITY: usize = 100;
        const LARGE_CAPACITY: usize = 1_000;

        let small_arena = Arena::<u64, u64, small_unsigned!(SMALL_CAPACITY), SMALL_CAPACITY>::new();
        let large_arena = Arena::<u64, u64, small_unsigned!(LARGE_CAPACITY), LARGE_CAPACITY>::new();

        let small_arena_size = size_of_val(&small_arena);
        let large_arena_size = size_of_val(&large_arena);

        println!("\nArena sizes:");
        println!("\tSmall: {} bytes", small_arena_size);
        println!("\tBig: {} bytes", large_arena_size);

        assert!(small_arena_size < large_arena_size);

        /*
        NOTE: This is draft code for upgrades when `feature(generic_const_exprs)` stabilizes.

        let small_node_size = small_arena.node_size();
        let large_node_size = large_arena.node_size();

        println!("\nNode sizes:");
        println!("\tSmall: {} bytes", small_node_size);
        println!("\tBig: {} bytes", large_node_size);

        assert!(small_node_size < large_node_size);
        */
    }
}
