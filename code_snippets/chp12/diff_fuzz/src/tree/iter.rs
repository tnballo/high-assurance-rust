use tinyvec::ArrayVec;

use super::node::Node;
use super::node_dispatch::SmallNode;
use super::tree::{Idx, SgTree};

// Immutable Reference Iterator ----------------------------------------------------------------------------------------

/// Uses iterative in-order tree traversal algorithm.
/// Maintains a small stack of arena indexes (won't contain all indexes simultaneously for a balanced tree).
pub struct Iter<'a, K: Default, V: Default, const N: usize> {
    bst: &'a SgTree<K, V, N>,
    idx_stack: ArrayVec<[usize; N]>,
    total_cnt: usize,
    spent_cnt: usize,
}

impl<'a, K: Ord + Default, V: Default, const N: usize> Iter<'a, K, V, N> {
    pub fn new(bst: &'a SgTree<K, V, N>) -> Self {
        let mut ordered_iter = Iter {
            bst,
            idx_stack: ArrayVec::<[usize; N]>::new(),
            total_cnt: bst.len(),
            spent_cnt: 0,
        };

        if let Some(root_idx) = ordered_iter.bst.opt_root_idx {
            let mut curr_idx = root_idx;
            loop {
                let node = &ordered_iter.bst.arena[curr_idx];
                match node.left_idx() {
                    Some(lt_idx) => {
                        ordered_iter.idx_stack.push(curr_idx);
                        curr_idx = lt_idx;
                    }
                    None => {
                        ordered_iter.idx_stack.push(curr_idx);
                        break;
                    }
                }
            }
        }

        ordered_iter
    }
}

impl<'a, K: Ord + Default, V: Default, const N: usize> Iterator for Iter<'a, K, V, N> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.idx_stack.pop() {
            Some(pop_idx) => {
                let node = &self.bst.arena[pop_idx];
                if let Some(gt_idx) = node.right_idx() {
                    let mut curr_idx = gt_idx;
                    loop {
                        let node = &self.bst.arena[curr_idx];
                        match node.left_idx() {
                            Some(lt_idx) => {
                                self.idx_stack.push(curr_idx);
                                curr_idx = lt_idx;
                            }
                            None => {
                                self.idx_stack.push(curr_idx);
                                break;
                            }
                        }
                    }
                }

                let node = &self.bst.arena[pop_idx];
                self.spent_cnt += 1;
                Some((node.key(), node.val()))
            }
            None => None,
        }
    }
}

impl<'a, K: Ord + Default, V: Default, const N: usize> ExactSizeIterator for Iter<'a, K, V, N> {
    fn len(&self) -> usize {
        debug_assert!(self.spent_cnt <= self.total_cnt);
        self.total_cnt - self.spent_cnt
    }
}

// Mutable Reference Iterator ------------------------------------------------------------------------------------------

pub struct IterMut<'a, K, V, const N: usize> {
    arena_iter_mut: core::slice::IterMut<'a, Option<Node<K, V, Idx>>>,
}

impl<'a, K: Ord + Default, V: Default, const N: usize> IterMut<'a, K, V, N> {
    pub fn new(bst: &'a mut SgTree<K, V, N>) -> Self {
        bst.sort_arena();
        IterMut {
            arena_iter_mut: bst.arena.iter_mut(),
        }
    }
}

impl<'a, K: Ord + Default, V: Default, const N: usize> Iterator for IterMut<'a, K, V, N> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.arena_iter_mut.next() {
            Some(Some(node)) => Some(node.get_mut()),
            _ => None,
        }
    }
}

impl<'a, K: Ord + Default, V: Default, const N: usize> ExactSizeIterator for IterMut<'a, K, V, N> {
    fn len(&self) -> usize {
        self.arena_iter_mut.len()
    }
}

// Consuming Iterator --------------------------------------------------------------------------------------------------

/// Cheats a little by using internal flattening logic to sort, instead of re-implementing proper traversal.
/// Maintains a shrinking list of arena indexes, initialized with all of them.
pub struct IntoIter<K: Default, V: Default, const N: usize> {
    bst: SgTree<K, V, N>,
    sorted_idxs: ArrayVec<[usize; N]>,
}

impl<K: Ord + Default, V: Default, const N: usize> IntoIter<K, V, N> {
    pub fn new(bst: SgTree<K, V, N>) -> Self {
        let mut ordered_iter = IntoIter {
            bst,
            sorted_idxs: ArrayVec::<[usize; N]>::new(),
        };

        if let Some(root_idx) = ordered_iter.bst.opt_root_idx {
            ordered_iter.sorted_idxs = ordered_iter.bst.flatten_subtree_to_sorted_idxs(root_idx);
            ordered_iter.sorted_idxs.reverse();
        }

        ordered_iter
    }
}

impl<K: Ord + Default, V: Default, const N: usize> Iterator for IntoIter<K, V, N> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.sorted_idxs.pop() {
            Some(idx) => match self.bst.priv_remove_by_idx(idx) {
                Some((key, val)) => Some((key, val)),
                None => {
                    debug_assert!(false, "Use of invalid index in consuming iterator!");
                    None
                }
            },
            None => None,
        }
    }
}

impl<K: Ord + Default, V: Default, const N: usize> ExactSizeIterator for IntoIter<K, V, N> {
    fn len(&self) -> usize {
        self.sorted_idxs.len()
    }
}
