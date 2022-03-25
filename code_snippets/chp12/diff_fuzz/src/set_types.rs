use core::cmp::Ordering;

use crate::set::SgSet;
use crate::tree::{Idx, IntoIter as TreeIntoIter, Iter as TreeIter};

use smallnum::SmallUnsigned;
use tinyvec::{ArrayVec, ArrayVecIterator};

// General Iterators ---------------------------------------------------------------------------------------------------

/// An iterator over the items of a [`SgSet`][crate::set::SgSet].
///
/// This `struct` is created by the [`iter`][crate::set::SgSet::iter] method on [`SgSet`][crate::set::SgSet].
/// See its documentation for more.
pub struct Iter<'a, T: Ord + Default, const N: usize> {
    ref_iter: TreeIter<'a, T, (), N>,
}

impl<'a, T: Ord + Default, const N: usize> Iter<'a, T, N> {
    /// Construct reference iterator.
    pub(crate) fn new(set: &'a SgSet<T, N>) -> Self {
        Iter {
            ref_iter: TreeIter::new(&set.bst),
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.ref_iter.next().map(|(k, _)| k)
    }
}

impl<'a, T: Ord + Default, const N: usize> ExactSizeIterator for Iter<'a, T, N> {
    fn len(&self) -> usize {
        self.ref_iter.len()
    }
}

/// An owning iterator over the items of a [`SgSet`][crate::set::SgSet].
///
/// This `struct` is created by the [`into_iter`][crate::set::SgSet::into_iter] method on [`SgSet`][crate::set::SgSet]
/// (provided by the IntoIterator trait). See its documentation for more.
pub struct IntoIter<T: Ord + Default, const N: usize> {
    cons_iter: TreeIntoIter<T, (), N>,
}

impl<T: Ord + Default, const N: usize> IntoIter<T, N> {
    /// Construct owning iterator.
    pub(crate) fn new(set: SgSet<T, N>) -> Self {
        IntoIter {
            cons_iter: TreeIntoIter::new(set.bst),
        }
    }
}

impl<T: Ord + Default, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.cons_iter.next().map(|(k, _)| k)
    }
}

impl<T: Ord + Default, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    fn len(&self) -> usize {
        self.cons_iter.len()
    }
}

/*
Workaround Note:

The remaining iterators in this file only store indexes into the input set(s) iterator(s) and have to
recover set elements with `set.iter().nth(idx)`. Rather inefficient, solves a blocking problem:
in `ArrayVecIterator<[&'a T; N]>` `Default` is not implemented for `&'a T`.

TODO: faster solution?
*/

// TODO: without `feature(generic_const_exprs)`, `Union` and `SymmetricDifference` cannot compute `2 * N` length
// iterator to support disjoint sets. This is a temporary workaround, documented in external API docs.
const PLACEHOLDER_2N: usize = 4096;

// Intersection Iterator -----------------------------------------------------------------------------------------------

// TODO: these need more trait implementations for full compatibility
// TODO: make this a lazy iterator like `std::collections::btree_set::Intersection`

/// An iterator producing elements in the intersection of [`SgSet`][crate::set::SgSet]s.
///
/// This `struct` is created by the [`intersection`][crate::set::SgSet::difference] method on [`SgSet`][crate::set::SgSet].
/// See its documentation for more.
pub struct Intersection<'a, T: Ord + Default, const N: usize> {
    pub(crate) inner: ArrayVecIterator<[Idx; N]>,
    set_this: &'a SgSet<T, N>,
    total_cnt: usize,
    spent_cnt: usize,
}

impl<'a, T: Ord + Default, const N: usize> Intersection<'a, T, N> {
    /// Construct `Intersection` iterator.
    /// Values that are both in `this` and `other`.
    pub(crate) fn new(this: &'a SgSet<T, N>, other: &SgSet<T, N>) -> Self {
        let mut self_enum_iter = this.iter().enumerate();
        let mut other_enum_iter = other.iter().enumerate();

        let mut opt_self = self_enum_iter.next();
        let mut opt_other = other_enum_iter.next();

        let mut inter = ArrayVec::default();
        let mut len = 0;

        // If either is shorter, short-circuit.
        while let (Some((self_idx, self_val)), Some((_, other_val))) = (opt_self, opt_other) {
            match self_val.cmp(other_val) {
                Ordering::Less => {
                    opt_self = self_enum_iter.next();
                }
                Ordering::Equal => {
                    inter.push(Idx::checked_from(self_idx));
                    len += 1;
                    opt_self = self_enum_iter.next();
                    opt_other = other_enum_iter.next();
                }
                Ordering::Greater => {
                    opt_other = other_enum_iter.next();
                }
            }
        }

        Intersection {
            inner: inter.into_iter(),
            set_this: this,
            total_cnt: len,
            spent_cnt: 0,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> Iterator for Intersection<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.inner.next() {
            Some(idx) => match self.set_this.iter().nth(idx.usize()) {
                Some(item) => {
                    self.spent_cnt += 1;
                    Some(item)
                }
                None => None,
            },
            None => None,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> ExactSizeIterator for Intersection<'a, T, N> {
    fn len(&self) -> usize {
        debug_assert!(self.spent_cnt <= self.total_cnt);
        self.total_cnt - self.spent_cnt
    }
}

// Difference Iterator -------------------------------------------------------------------------------------------------

// TODO: these need more trait implementations for full compatibility
// TODO: make this a lazy iterator like `std::collections::btree_set::Difference`

/// An iterator producing elements in the difference of [`SgSet`][crate::set::SgSet]s.
///
/// This `struct` is created by the [`difference`][crate::set::SgSet::difference] method
/// on [`SgSet`][crate::set::SgSet]. See its documentation for more.
pub struct Difference<'a, T: Ord + Default, const N: usize> {
    pub(crate) inner: ArrayVecIterator<[Idx; N]>,
    set_this: &'a SgSet<T, N>,
    total_cnt: usize,
    spent_cnt: usize,
}

impl<'a, T: Ord + Default, const N: usize> Difference<'a, T, N> {
    /// Construct `Difference` iterator.
    /// Values that are in `this` but not in `other`.
    pub(crate) fn new(this: &'a SgSet<T, N>, other: &SgSet<T, N>) -> Self {
        let mut diff = ArrayVec::default();
        let mut len = 0;

        for (idx, val) in this.iter().enumerate() {
            if !other.contains(val) {
                diff.push(Idx::checked_from(idx));
                len += 1;
            }
        }

        Difference {
            inner: diff.into_iter(),
            set_this: this,
            total_cnt: len,
            spent_cnt: 0,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> Iterator for Difference<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.inner.next() {
            Some(idx) => match self.set_this.iter().nth(idx.usize()) {
                Some(item) => {
                    self.spent_cnt += 1;
                    Some(item)
                }
                None => None,
            },
            None => None,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> ExactSizeIterator for Difference<'a, T, N> {
    fn len(&self) -> usize {
        debug_assert!(self.spent_cnt <= self.total_cnt);
        self.total_cnt - self.spent_cnt
    }
}

// Symmetric Difference Iterator ---------------------------------------------------------------------------------------

// TODO: these need more trait implementations for full compatibility
// TODO: make this a lazy iterator like `std::collections::btree_set::Difference`

/// An iterator producing elements in the symmetric difference of [`SgSet`][crate::set::SgSet]s.
///
/// This `struct` is created by the [`symmetric_difference`][crate::set::SgSet::symmetric_difference]
/// method on [`SgSet`][crate::set::SgSet]. See its documentation for more.
pub struct SymmetricDifference<'a, T: Ord + Default, const N: usize> {
    pub(crate) inner: ArrayVecIterator<[(Idx, bool); PLACEHOLDER_2N]>, // TODO: placeholder
    set_this: &'a SgSet<T, N>,
    set_other: &'a SgSet<T, N>,
    total_cnt: usize,
    spent_cnt: usize,
}

impl<'a, T: Ord + Default, const N: usize> SymmetricDifference<'a, T, N> {
    /// Construct `SymmetricDifference` iterator.
    /// Values that are in `this` or in `other` but not in both.
    pub(crate) fn new(this: &'a SgSet<T, N>, other: &'a SgSet<T, N>) -> Self {
        let mut sym_diff = ArrayVec::default();
        let mut len = 0;

        for (idx, val) in this.iter().enumerate() {
            if !other.contains(val) {
                sym_diff.push((Idx::checked_from(idx), true));
                len += 1;
            }
        }

        for (idx, val) in other.iter().enumerate() {
            if !this.contains(val) {
                sym_diff.push((Idx::checked_from(idx), false));
                len += 1;
            }
        }

        // Ascending order
        sym_diff.sort_unstable_by_key(|(idx, in_this): &(Idx, bool)| match in_this {
            true => this.iter().nth(idx.usize()),
            false => other.iter().nth(idx.usize()),
        });

        SymmetricDifference {
            inner: sym_diff.into_iter(),
            set_this: this,
            set_other: other,
            total_cnt: len,
            spent_cnt: 0,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> Iterator for SymmetricDifference<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.inner.next() {
            Some((idx, in_this)) => match in_this {
                true => match self.set_this.iter().nth(idx.usize()) {
                    Some(item) => {
                        self.spent_cnt += 1;
                        Some(item)
                    }
                    None => None,
                },
                false => match self.set_other.iter().nth(idx.usize()) {
                    Some(item) => {
                        self.spent_cnt += 1;
                        Some(item)
                    }
                    None => None,
                },
            },
            None => None,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> ExactSizeIterator for SymmetricDifference<'a, T, N> {
    fn len(&self) -> usize {
        debug_assert!(self.spent_cnt <= self.total_cnt);
        self.total_cnt - self.spent_cnt
    }
}

// Union Iterator ------------------------------------------------------------------------------------------------------

// TODO: these need more trait implementations for full compatibility
// TODO: make this a lazy iterator like `std::collections::btree_set::Union`

/// An iterator producing elements in the union of [`SgSet`][crate::set::SgSet]s.
///
/// This `struct` is created by the [`union`][crate::set::SgSet::difference] method on [`SgSet`][crate::set::SgSet].
/// See its documentation for more.
pub struct Union<'a, T: Ord + Default, const N: usize> {
    pub(crate) inner: ArrayVecIterator<[(Idx, bool); PLACEHOLDER_2N]>,
    set_this: &'a SgSet<T, N>,
    set_other: &'a SgSet<T, N>,
    total_cnt: usize,
    spent_cnt: usize,
}

impl<'a, T: Ord + Default, const N: usize> Union<'a, T, N> {
    /// Construct `Union` iterator.
    /// Values in `this` or `other`, without duplicates.
    pub(crate) fn new(this: &'a SgSet<T, N>, other: &'a SgSet<T, N>) -> Self {
        let mut uni = ArrayVec::default();
        let mut len = 0;

        for (idx, _) in this.iter().enumerate() {
            uni.push((Idx::checked_from(idx), true));
            len += 1;
        }

        for (idx, val) in other.iter().enumerate() {
            if !this.contains(val) {
                uni.push((Idx::checked_from(idx), false));
                len += 1;
            }
        }

        // Ascending order
        uni.sort_unstable_by_key(|(idx, in_this): &(Idx, bool)| match in_this {
            true => this.iter().nth(idx.usize()),
            false => other.iter().nth(idx.usize()),
        });

        Union {
            inner: uni.into_iter(),
            set_this: this,
            set_other: other,
            total_cnt: len,
            spent_cnt: 0,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> Iterator for Union<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.inner.next() {
            Some((idx, in_this)) => match in_this {
                true => match self.set_this.iter().nth(idx.usize()) {
                    Some(item) => {
                        self.spent_cnt += 1;
                        Some(item)
                    }
                    None => None,
                },
                false => match self.set_other.iter().nth(idx.usize()) {
                    Some(item) => {
                        self.spent_cnt += 1;
                        Some(item)
                    }
                    None => None,
                },
            },
            None => None,
        }
    }
}

impl<'a, T: Ord + Default, const N: usize> ExactSizeIterator for Union<'a, T, N> {
    fn len(&self) -> usize {
        debug_assert!(self.spent_cnt <= self.total_cnt);
        self.total_cnt - self.spent_cnt
    }
}
