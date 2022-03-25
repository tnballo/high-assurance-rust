use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::iter::FromIterator;
use core::ops::{BitAnd, BitOr, BitXor, Sub};

use crate::set_types::{Difference, Intersection, IntoIter, Iter, SymmetricDifference, Union};
use crate::tree::{SgError, SgTree};

/// Safe, fallible, embedded-friendly ordered set.
///
/// ### Fallible APIs
///
/// * [`try_insert`][crate::set::SgSet::try_insert]
/// * [`try_append`][crate::set::SgSet::try_append]
/// * [`try_extend`][crate::set::SgSet::try_extend]
/// * [`try_from_iter`][crate::set::SgSet::try_from_iter]
/// * [`try_replace`][crate::set::SgSet::try_replace]
///
/// [`TryFrom`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html) isn't implemented because it would collide with the blanket implementation.
/// See [this open GitHub issue](https://github.com/rust-lang/rust/issues/50133#issuecomment-64690839) from 2018,
/// this is a known Rust limitation that should be fixed via specialization in the future.
///
/// ### Attribution Note
///
/// The majority of API examples and descriptions are adapted or directly copied from the standard library's [`BTreeSet`](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html).
/// The goal is to offer embedded developers familiar, ergonomic APIs on resource constrained systems that otherwise don't get the luxury of dynamic collections.
#[derive(Default, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct SgSet<T: Ord + Default, const N: usize> {
    pub(crate) bst: SgTree<T, (), N>,
}

impl<T: Ord + Default, const N: usize> SgSet<T, N> {
    /// Makes a new, empty `SgSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set: SgSet<i32, 10> = SgSet::new();
    /// ```
    pub fn new() -> Self {
        SgSet { bst: SgTree::new() }
    }

    /// The [original scapegoat tree paper's](https://people.csail.mit.edu/rivest/pubs/GR93.pdf) alpha, `a`, can be chosen in the range `0.5 <= a < 1.0`.
    /// `a` tunes how "aggressively" the data structure self-balances.
    /// It controls the trade-off between total rebuild time and maximum height guarantees.
    ///
    /// * As `a` approaches `0.5`, the tree will rebalance more often. Ths means slower insertions, but faster lookups and deletions.
    ///     * An `a` equal to `0.5` means a tree that always maintains a perfect balance (e.g."complete" binary tree, at all times).
    ///
    /// * As `a` approaches `1.0`, the tree will rebalance less often. This means quicker insertions, but slower lookups and deletions.
    ///     * If `a` reached `1.0`, it'd mean a tree that never rebalances.
    ///
    /// Returns `Err` if `0.5 <= alpha_num / alpha_denom < 1.0` isn't `true` (invalid `a`, out of range).
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set: SgSet<isize, 10> = SgSet::new();
    ///
    /// // Set 2/3, e.g. `a = 0.666...` (it's default value).
    /// assert!(set.set_rebal_param(2.0, 3.0).is_ok());
    /// ```
    #[doc(alias = "rebalance")]
    #[doc(alias = "alpha")]
    pub fn set_rebal_param(&mut self, alpha_num: f32, alpha_denom: f32) -> Result<(), SgError> {
        self.bst.set_rebal_param(alpha_num, alpha_denom)
    }

    /// Get the current rebalance parameter, alpha, as a tuple of `(alpha_numerator, alpha_denominator)`.
    /// See [the corresponding setter method][SgSet::set_rebal_param] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set: SgSet<isize, 10> = SgSet::new();
    ///
    /// // Set 2/3, e.g. `a = 0.666...` (it's default value).
    /// assert!(set.set_rebal_param(2.0, 3.0).is_ok());
    ///
    /// // Get the currently set value
    /// assert_eq!(set.rebal_param(), (2.0, 3.0));
    /// ```
    #[doc(alias = "rebalance")]
    #[doc(alias = "alpha")]
    pub fn rebal_param(&self) -> (f32, f32) {
        self.bst.rebal_param()
    }

    /// Total capacity, e.g. maximum number of set elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set: SgSet<i32, 10> = SgSet::new();
    ///
    /// assert!(set.capacity() == 10)
    /// ```
    pub fn capacity(&self) -> usize {
        self.bst.capacity()
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// b.insert(3);
    /// b.insert(4);
    /// b.insert(5);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert!(a.contains(&1));
    /// assert!(a.contains(&2));
    /// assert!(a.contains(&3));
    /// assert!(a.contains(&4));
    /// assert!(a.contains(&5));
    /// ```
    pub fn append(&mut self, other: &mut SgSet<T, N>)
    where
        T: Ord,
    {
        self.bst.append(&mut other.bst);
    }

    /// Attempts to move all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::iter::FromIterator;
    /// use buggy_scapegoat::{SgSet, SgError};
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// assert!(a.try_insert(1).is_ok());
    /// assert!(a.try_insert(2).is_ok());
    /// assert!(a.try_insert(3).is_ok());
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// assert!(b.try_insert(3).is_ok()); // Overwrite previous
    /// assert!(b.try_insert(4).is_ok());
    /// assert!(b.try_insert(5).is_ok());
    ///
    /// // Successful append
    /// assert!(a.try_append(&mut b).is_ok());
    ///
    /// // Elements moved
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert!(a.contains(&1));
    /// assert!(a.contains(&2));
    /// assert!(a.contains(&3));
    /// assert!(a.contains(&4));
    /// assert!(a.contains(&5));
    ///
    /// // Fill remaining capacity
    /// let mut key = 6;
    /// while a.len() < a.capacity() {
    ///     assert!(a.try_insert(key).is_ok());
    ///     key += 1;
    /// }
    ///
    /// // Full
    /// assert!(a.is_full());
    ///
    /// // More data
    /// let mut c = SgSet::<_, 10>::from_iter([11, 12]);
    /// let mut d = SgSet::<_, 10>::from_iter([1, 2]);
    ///
    /// // Cannot append new pairs
    /// assert_eq!(a.try_append(&mut c), Err(SgError::StackCapacityExceeded));
    ///
    /// // Can still replace existing pairs
    /// assert!(a.try_append(&mut d).is_ok());
    /// ```
    pub fn try_append(&mut self, other: &mut SgSet<T, N>) -> Result<(), SgError> {
        self.bst.try_append(&mut other.bst)
    }

    /// Adds a value to the set.
    /// If the set did not have this value present, `true` is returned.
    /// If the set did have this value present, `false` is returned, and the entry is overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, value: T) -> bool
    where
        T: Ord,
    {
        self.bst.insert(value, ()).is_none()
    }

    /// Adds a value to the set.
    /// Returns `Err` if the operation can't be completed, else the `Ok` contains:
    /// * `true` if the set did not have this value present.
    /// * `false` if the set did have this value present (and that old entry is overwritten).
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::{SgSet, SgError};
    ///
    /// let mut set = SgSet::<_, 10>::new();
    ///
    /// // Add a new element
    /// assert_eq!(set.try_insert(2), Ok(true));
    ///
    /// // Replace existing element
    /// assert_eq!(set.try_insert(2), Ok(false));
    /// assert_eq!(set.len(), 1);
    ///
    /// // Fill remaining capacity
    /// let mut elem = 3;
    /// while set.len() < set.capacity() {
    ///     assert!(set.try_insert(elem).is_ok());
    ///     elem += 1;
    /// }
    ///
    /// // Full
    /// assert!(set.is_full());
    ///
    /// // Cannot insert new element
    /// assert_eq!(set.try_insert(elem), Err(SgError::StackCapacityExceeded));
    ///
    /// // Can still replace existing element
    /// assert_eq!(set.try_insert(elem - 1), Ok(false));
    /// ```
    pub fn try_insert(&mut self, value: T) -> Result<bool, SgError>
    where
        T: Ord,
    {
        match self.bst.try_insert(value, ()) {
            Ok(opt_val) => Ok(opt_val.is_none()),
            Err(_) => Err(SgError::StackCapacityExceeded),
        }
    }

    /// Attempt to extend a collection with the contents of an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::iter::FromIterator;
    /// use buggy_scapegoat::{SgSet, SgError};
    ///
    /// let mut a = SgSet::<_, 2>::new();
    /// let mut b = SgSet::<_, 3>::from_iter([1, 2, 3]);
    /// let mut c = SgSet::<_, 2>::from_iter([1, 2]);
    ///
    /// // Too big
    /// assert_eq!(a.try_extend(b.into_iter()), Err(SgError::StackCapacityExceeded));
    ///
    /// // Fits
    /// assert!(a.try_extend(c.into_iter()).is_ok());
    /// ```
    ///
    /// ### Note
    ///
    /// There is no `TryExtend` trait in `core`/`std`.
    pub fn try_extend<I: ExactSizeIterator + IntoIterator<Item = T>>(
        &mut self,
        iter: I,
    ) -> Result<(), SgError> {
        // Derp :P
        if iter.len() <= (self.capacity() - self.len()) {
            let map: crate::SgMap<T, (), N> = iter.into_iter().map(|e| (e, ())).collect();
            self.bst.try_extend(map.into_iter())
        } else {
            Err(SgError::StackCapacityExceeded)
        }
    }

    /// Attempt conversion from an iterator.
    /// Will fail if iterator length exceeds `u16::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::{SgSet, SgError};
    ///
    /// const CAPACITY_1: usize = 1_000;
    /// assert!(SgSet::<_, CAPACITY_1>::try_from_iter((0..CAPACITY_1)).is_ok());
    ///
    /// const CAPACITY_2: usize = (u16::MAX as usize) + 1;
    /// assert_eq!(
    ///     SgSet::<_, CAPACITY_2>::try_from_iter((0..CAPACITY_2)),
    ///     Err(SgError::MaximumCapacityExceeded)
    /// );
    /// ```
    ///
    /// ### Note
    ///
    /// There is no `TryFromIterator` trait in `core`/`std`.
    pub fn try_from_iter<I: ExactSizeIterator + IntoIterator<Item = T>>(
        iter: I,
    ) -> Result<Self, SgError> {
        match iter.len() <= SgTree::<T, (), N>::max_capacity() {
            true => Ok(SgSet::from_iter(iter)),
            false => Err(SgError::MaximumCapacityExceeded),
        }
    }

    /// Gets an iterator that visits the values in the `SgSet` in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let set: SgSet<usize, 3> = [1, 2, 3].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let set: SgSet<usize, 3> = [3, 1, 2].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter::new(self)
    }

    /// Removes a value from the set. Returns whether the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.remove(value).is_some()
    }

    /// Splits the collection into two at the given value. Returns everything after the given value,
    /// including the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 5>::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// a.insert(17);
    /// a.insert(41);
    ///
    /// let b = a.split_off(&3);
    ///
    /// assert_eq!(a.len(), 2);
    /// assert_eq!(b.len(), 3);
    ///
    /// assert!(a.contains(&1));
    /// assert!(a.contains(&2));
    ///
    /// assert!(b.contains(&3));
    /// assert!(b.contains(&17));
    /// assert!(b.contains(&41));
    /// ```
    pub fn split_off<Q>(&mut self, value: &Q) -> SgSet<T, N>
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        SgSet {
            bst: self.bst.split_off(value),
        }
    }

    /// Adds a value to the set, replacing the existing value, if any, that is equal to the given
    /// one. Returns the replaced value.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    /// set.insert(Vec::<i32>::new());
    ///
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 0);
    /// set.replace(Vec::with_capacity(10));
    /// assert_eq!(set.get(&[][..]).unwrap().capacity(), 10);
    /// ```
    pub fn replace(&mut self, value: T) -> Option<T>
    where
        T: Ord,
    {
        let removed = self.bst.remove_entry(&value).map(|(k, _)| k);
        self.insert(value);
        removed
    }

    // TODO: add example
    /// Attempts to add a value to the set, replacing the existing value, if any, that is equal to the given
    /// one. Returns the replaced value.
    pub fn try_replace(&mut self, value: T) -> Result<Option<T>, SgError>
    where
        T: Ord,
    {
        let removed = self.bst.remove_entry(&value).map(|(k, _)| k);
        self.try_insert(value)?;
        Ok(removed)
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set: SgSet<_, 10> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.take(&2), Some(2));
    /// assert_eq!(set.take(&2), None);
    /// ```
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.remove_entry(value).map(|(k, _)| k)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// The elements are visited in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let xs = [1, 2, 3, 4, 5, 6];
    /// let mut set: SgSet<i32, 10> = xs.iter().cloned().collect();
    /// // Keep only the even numbers.
    /// set.retain(|&k| k % 2 == 0);
    /// assert!(set.iter().eq([2, 4, 6].iter()));
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        T: Ord,
        F: FnMut(&T) -> bool,
    {
        self.bst.retain(|k, _| f(k));
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let set: SgSet<_, 10> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.get(&2), Some(&2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.get_key_value(value).map(|(k, _)| k)
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut v = SgSet::<_, 10>::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());;
    /// ```
    pub fn clear(&mut self) {
        self.bst.clear()
    }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let set: SgSet<_, 10> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.contains_key(value)
    }

    /// Returns a reference to the first/minium value in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 2>::new();
    /// assert_eq!(set.first(), None);
    /// set.insert(1);
    /// assert_eq!(set.first(), Some(&1));
    /// set.insert(2);
    /// assert_eq!(set.first(), Some(&1));
    /// ```
    pub fn first(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.bst.first_key()
    }

    /// Removes the first value from the set and returns it, if any.
    /// The first value is the minimum value that was in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_first() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_first(&mut self) -> Option<T>
    where
        T: Ord,
    {
        self.bst.pop_first().map(|(k, _)| k)
    }

    /// Returns the last/maximum value in the set, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    /// assert_eq!(set.first(), None);
    /// set.insert(1);
    /// assert_eq!(set.last(), Some(&1));
    /// set.insert(2);
    /// assert_eq!(set.last(), Some(&2));
    /// ```
    pub fn last(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.bst.last_key()
    }

    /// Removes the last value from the set and returns it, if any.
    /// The last value is the maximum value that was in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut set = SgSet::<_, 10>::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_last() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<T>
    where
        T: Ord,
    {
        self.bst.pop_last().map(|(k, _)| k)
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut v = SgSet::<_, 10>::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.bst.len()
    }

    /// Returns an iterator over values representing set difference, e.g., values in `self` but not in `other`, in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let diff: Vec<_> = a.difference(&b).cloned().collect();
    /// assert_eq!(diff, [1]);
    /// ```
    pub fn difference(&self, other: &SgSet<T, N>) -> Difference<T, N>
    where
        T: Ord,
    {
        Difference::new(self, other)
    }

    /// Returns an iterator over values representing symmetric set difference, e.g., values in `self` or `other` but not both, in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let sym_diff: Vec<_> = a.symmetric_difference(&b).cloned().collect();
    /// assert_eq!(sym_diff, [1, 3]);
    /// ```
    ///
    /// ### Warning
    ///
    /// At present, this function may panic if set capacity `N` exceeds `2048`.
    /// The issue is that this function's returned iterator needs to be `2 * N` long to support disjoint sets,
    /// but without unstable `feature(generic_const_exprs)` we can't compute `2 * N`.
    /// So we use `4096` instead of `2 * N` as a workaround, hence `N` should be `<= 2048` to ensure no panic.
    /// An `N > 2048` may or may not panic, depending on the size of sets' intersection.
    pub fn symmetric_difference<'a>(&'a self, other: &'a SgSet<T, N>) -> SymmetricDifference<T, N>
    where
        T: Ord,
    {
        SymmetricDifference::new(self, other)
    }

    /// Returns an iterator over values representing set intersection, e.g., values in both `self` and `other`, in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let intersection: Vec<_> = a.intersection(&b).cloned().collect();
    /// assert_eq!(intersection, [2]);
    /// ```
    pub fn intersection(&self, other: &SgSet<T, N>) -> Intersection<T, N>
    where
        T: Ord,
    {
        Intersection::new(self, other)
    }

    /// Returns an iterator over values representing set union, e.g., values in `self` or `other`, in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 10>::new();
    /// a.insert(1);
    ///
    /// let mut b = SgSet::<_, 10>::new();
    /// b.insert(2);
    ///
    /// let union: Vec<_> = a.union(&b).cloned().collect();
    /// assert_eq!(union, [1, 2]);
    /// ```
    ///
    /// ### Warning
    ///
    /// At present, this function may panic if set capacity `N` exceeds `2048`.
    /// The issue is that this function's returned iterator needs to be `2 * N` long to support disjoint sets,
    /// but without unstable `feature(generic_const_exprs)` we can't compute `2 * N`.
    /// So we use `4096` instead of `2 * N` as a workaround, hence `N` should be `<= 2048` to ensure no panic.
    /// An `N > 2048` may or may not panic, depending on the size of sets' intersection.
    pub fn union<'a>(&'a self, other: &'a SgSet<T, N>) -> Union<T, N>
    where
        T: Ord,
    {
        Union::new(self, other)
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut v = SgSet::<_, 10>::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bst.is_empty()
    }

    /// Returns `true` if the set's capacity is filled.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let mut a = SgSet::<_, 2>::new();
    /// a.insert(1);
    /// assert!(!a.is_full());
    /// a.insert(2);
    /// assert!(a.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.bst.is_full()
    }

    /// Returns `true` if `self` has no elements in common with other (empty intersection).
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    /// let a: SgSet<_, 10> = [1, 2, 3].iter().cloned().collect();
    /// let mut b = SgSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    pub fn is_disjoint(&self, other: &SgSet<T, N>) -> bool
    where
        T: Ord,
    {
        self.intersection(other).count() == 0
    }

    /// Returns `true` if `self` is a subset of `other`, e.g., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let sup: SgSet<_, 10> = [1, 2, 3].iter().cloned().collect();
    /// let mut set = SgSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    pub fn is_subset(&self, other: &SgSet<T, N>) -> bool
    where
        T: Ord,
    {
        self.intersection(other).count() == self.len()
    }

    /// Returns `true` if `self` is a superset of `other`, e.g., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let sub: SgSet<_, 3> = [1, 2].iter().cloned().collect();
    /// let mut set = SgSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    pub fn is_superset(&self, other: &SgSet<T, N>) -> bool
    where
        T: Ord,
    {
        other.is_subset(self)
    }
}

// Convenience Traits --------------------------------------------------------------------------------------------------

// Debug
impl<T, const N: usize> Debug for SgSet<T, N>
where
    T: Ord + Default + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set()
            .entries(self.bst.iter().map(|(k, _)| k))
            .finish()
    }
}

// From array.
impl<T, const N: usize> From<[T; N]> for SgSet<T, N>
where
    T: Ord + Default,
{
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let set1 = SgSet::from([1, 2, 3, 4]);
    /// let set2: SgSet<_, 4> = [1, 2, 3, 4].into();
    /// assert_eq!(set1, set2);
    /// ```
    ///
    /// ### Warning
    ///
    /// [`TryFrom`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html) isn't implemented because it would collide with the blanket implementation.
    /// See [this open GitHub issue](https://github.com/rust-lang/rust/issues/50133#issuecomment-64690839) from 2018,
    /// this is a known Rust limitation that should be fixed via specialization in the future.
    #[doc(alias = "tryfrom")]
    #[doc(alias = "try_from")]
    #[doc(alias = "TryFrom")]
    fn from(arr: [T; N]) -> Self {
        IntoIterator::into_iter(arr).collect()
    }
}

// Construct from iterator.
impl<T, const N: usize> FromIterator<T> for SgSet<T, N>
where
    T: Ord + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut sgs = SgSet::new();
        sgs.bst = SgTree::from_iter(iter.into_iter().map(|e| (e, ())));
        sgs
    }
}

// Extension from iterator.
impl<T, const N: usize> Extend<T> for SgSet<T, N>
where
    T: Ord + Default,
{
    fn extend<TreeIter: IntoIterator<Item = T>>(&mut self, iter: TreeIter) {
        self.bst.extend(iter.into_iter().map(|e| (e, ())));
    }
}

// Extension from reference iterator.
impl<'a, T, const N: usize> Extend<&'a T> for SgSet<T, N>
where
    T: 'a + Ord + Default + Copy,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

// General Iterators ---------------------------------------------------------------------------------------------------

// Reference iterator
impl<'a, T: Ord + Default, const N: usize> IntoIterator for &'a SgSet<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Consuming iterator
impl<T: Ord + Default, const N: usize> IntoIterator for SgSet<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

// Operator Overloading ------------------------------------------------------------------------------------------------

impl<T: Ord + Default + Clone, const N: usize> Sub<&SgSet<T, N>> for &SgSet<T, N> {
    type Output = SgSet<T, N>;

    /// Returns the difference of `self` and `rhs` as a new `SgSet<T, N>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let a: SgSet<_, 10> = vec![1, 2, 3].into_iter().collect();
    /// let b: SgSet<_, 10> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let result = &a - &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 2]);
    /// ```
    fn sub(self, rhs: &SgSet<T, N>) -> SgSet<T, N> {
        self.difference(rhs).cloned().collect()
    }
}

impl<T: Ord + Default + Clone, const N: usize> BitAnd<&SgSet<T, N>> for &SgSet<T, N> {
    type Output = SgSet<T, N>;

    /// Returns the intersection of `self` and `rhs` as a new `SgSet<T, N>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let a: SgSet<_, 10> = vec![1, 2, 3].into_iter().collect();
    /// let b: SgSet<_, 10> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let result = &a & &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [2, 3]);
    /// ```
    fn bitand(self, rhs: &SgSet<T, N>) -> SgSet<T, N> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T: Ord + Default + Clone, const N: usize> BitOr<&SgSet<T, N>> for &SgSet<T, N> {
    type Output = SgSet<T, N>;

    /// Returns the union of `self` and `rhs` as a new `SgSet<T, N>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let a: SgSet<_, 10> = vec![1, 2, 3].into_iter().collect();
    /// let b: SgSet<_, 10> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let result = &a | &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 2, 3, 4, 5]);
    /// ```
    fn bitor(self, rhs: &SgSet<T, N>) -> SgSet<T, N> {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Ord + Default + Clone, const N: usize> BitXor<&SgSet<T, N>> for &SgSet<T, N> {
    type Output = SgSet<T, N>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `SgSet<T, N>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgSet;
    ///
    /// let a: SgSet<_, 10> = vec![1, 2, 3].into_iter().collect();
    /// let b: SgSet<_, 10> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let result = &a ^ &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 4]);
    /// ```
    fn bitxor(self, rhs: &SgSet<T, N>) -> SgSet<T, N> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}
