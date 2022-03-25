use core::borrow::Borrow;
use core::fmt::{self, Debug};
use core::iter::FromIterator;
use core::ops::Index;

use crate::map_types::{
    Entry, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, OccupiedEntry, VacantEntry, Values,
    ValuesMut,
};
use crate::tree::{SgError, SgTree};

/// Safe, fallible, embedded-friendly ordered map.
///
/// ### Fallible APIs
///
/// * [`try_insert`][crate::map::SgMap::try_insert]
/// * [`try_append`][crate::map::SgMap::try_append]
/// * [`try_extend`][crate::map::SgMap::try_extend]
/// * [`try_from_iter`][crate::map::SgMap::try_from_iter]
///
/// [`TryFrom`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html) isn't implemented because it would collide with the blanket implementation.
/// See [this open GitHub issue](https://github.com/rust-lang/rust/issues/50133#issuecomment-64690839) from 2018,
/// this is a known Rust limitation that should be fixed via specialization in the future.
///
/// ### Attribution Note
///
/// The majority of API examples and descriptions are adapted or directly copied from the standard library's [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html).
/// The goal is to offer embedded developers familiar, ergonomic APIs on resource constrained systems that otherwise don't get the luxury of dynamic collections.
#[derive(Default, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct SgMap<K: Ord + Default, V: Default, const N: usize> {
    pub(crate) bst: SgTree<K, V, N>,
}

impl<K: Ord + Default, V: Default, const N: usize> SgMap<K, V, N> {
    /// Makes a new, empty `SgMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    ///
    /// map.insert(1, "a");
    /// ```
    pub fn new() -> Self {
        SgMap { bst: SgTree::new() }
    }

    /// The [original buggy_scapegoat tree paper's](https://people.csail.mit.edu/rivest/pubs/GR93.pdf) alpha, `a`, can be chosen in the range `0.5 <= a < 1.0`.
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
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map: SgMap<isize, isize, 10> = SgMap::new();
    ///
    /// // Set 2/3, e.g. `a = 0.666...` (it's default value).
    /// assert!(map.set_rebal_param(2.0, 3.0).is_ok());
    /// ```
    #[doc(alias = "rebalance")]
    #[doc(alias = "alpha")]
    pub fn set_rebal_param(&mut self, alpha_num: f32, alpha_denom: f32) -> Result<(), SgError> {
        self.bst.set_rebal_param(alpha_num, alpha_denom)
    }

    /// Get the current rebalance parameter, alpha, as a tuple of `(alpha_numerator, alpha_denominator)`.
    /// See [the corresponding setter method][SgMap::set_rebal_param] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map: SgMap<isize, isize, 10> = SgMap::new();
    ///
    /// // Set 2/3, e.g. `a = 0.666...` (it's default value).
    /// assert!(map.set_rebal_param(2.0, 3.0).is_ok());
    ///
    /// // Get the currently set value
    /// assert_eq!(map.rebal_param(), (2.0, 3.0));
    /// ```
    #[doc(alias = "rebalance")]
    #[doc(alias = "alpha")]
    pub fn rebal_param(&self) -> (f32, f32) {
        self.bst.rebal_param()
    }

    /// Total capacity, e.g. maximum number of map pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<usize, &str, 10>::new();
    ///
    /// assert!(map.capacity() == 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.bst.capacity()
    }

    /// Gets an iterator over the keys of the map, in sorted order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let keys: Vec<_> = a.keys().cloned().collect();
    /// assert_eq!(keys, [1, 2]);
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V, N> {
        Keys { inner: self.iter() }
    }

    /// Creates a consuming iterator visiting all the keys, in sorted order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let keys: Vec<i32> = a.into_keys().collect();
    /// assert_eq!(keys, [1, 2]);
    /// ```
    pub fn into_keys(self) -> IntoKeys<K, V, N> {
        IntoKeys {
            inner: self.into_iter(),
        }
    }

    /// Gets an iterator over the values of the map, in order by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, "hello");
    /// a.insert(2, "goodbye");
    ///
    /// let values: Vec<&str> = a.values().cloned().collect();
    /// assert_eq!(values, ["hello", "goodbye"]);
    /// ```
    pub fn values(&self) -> Values<'_, K, V, N> {
        Values { inner: self.iter() }
    }

    /// Creates a consuming iterator visiting all the values, in order by key.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, "hello");
    /// a.insert(2, "goodbye");
    ///
    /// let values: Vec<&str> = a.into_values().collect();
    /// assert_eq!(values, ["hello", "goodbye"]);
    /// ```
    pub fn into_values(self) -> IntoValues<K, V, N> {
        IntoValues {
            inner: self.into_iter(),
        }
    }

    /// Gets a mutable iterator over the values of the map, in order by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, String::from("hello"));
    /// a.insert(2, String::from("goodbye"));
    ///
    /// for value in a.values_mut() {
    ///     value.push_str("!");
    /// }
    ///
    /// let values: Vec<String> = a.values().cloned().collect();
    /// assert_eq!(values, [String::from("hello!"),
    ///                     String::from("goodbye!")]);
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V, N> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    ///
    /// let mut b = SgMap::<_, _, 10>::new();
    /// b.insert(3, "d");
    /// b.insert(4, "e");
    /// b.insert(5, "f");
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert_eq!(a[&1], "a");
    /// assert_eq!(a[&2], "b");
    /// assert_eq!(a[&3], "d");
    /// assert_eq!(a[&4], "e");
    /// assert_eq!(a[&5], "f");
    /// ```
    pub fn append(&mut self, other: &mut SgMap<K, V, N>) {
        self.bst.append(&mut other.bst);
    }

    /// Attempts to move all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::iter::FromIterator;
    /// use buggy_scapegoat::{SgMap, SgError};
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.try_insert(1, "a").is_ok();
    /// a.try_insert(2, "b").is_ok();
    /// a.try_insert(3, "c").is_ok();
    ///
    /// let mut b = SgMap::<_, _, 10>::new();
    /// b.try_insert(3, "d").is_ok(); // Overwrite previous
    /// b.try_insert(4, "e").is_ok();
    /// b.try_insert(5, "f").is_ok();
    ///
    /// // Successful append
    /// assert!(a.try_append(&mut b).is_ok());
    ///
    /// // Elements moved
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert_eq!(a[&1], "a");
    /// assert_eq!(a[&2], "b");
    /// assert_eq!(a[&3], "d");
    /// assert_eq!(a[&4], "e");
    /// assert_eq!(a[&5], "f");
    ///
    /// // Fill remaining capacity
    /// let mut key = 6;
    /// while a.len() < a.capacity() {
    ///     assert!(a.try_insert(key, "filler").is_ok());
    ///     key += 1;
    /// }
    ///
    /// // Full
    /// assert!(a.is_full());
    ///
    /// // More data
    /// let mut c = SgMap::<_, _, 10>::from_iter([(11, "k"), (12, "l")]);
    /// let mut d = SgMap::<_, _, 10>::from_iter([(1, "a2"), (2, "b2")]);
    ///
    /// // Cannot append new pairs
    /// assert_eq!(a.try_append(&mut c), Err(SgError::StackCapacityExceeded));
    ///
    /// // Can still replace existing pairs
    /// assert!(a.try_append(&mut d).is_ok());
    /// ```
    pub fn try_append(&mut self, other: &mut SgMap<K, V, N>) -> Result<(), SgError> {
        self.bst.try_append(&mut other.bst)
    }

    /// Insert a key-value pair into the map.
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, the old value is returned,
    /// and the key is updated. This accommodates types that can be `==` without being identical.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, val: V) -> Option<V>
    where
        K: Ord,
    {
        self.bst.insert(key, val)
    }

    /// Insert a key-value pair into the map.
    /// Returns `Err` if the operation can't be completed, else the `Ok` contains:
    /// * `None` if the map did not have this key present.
    /// * The old value if the map did have this key present (both the value and key are updated,
    /// this accommodates types that can be `==` without being identical).
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::{SgMap, SgError};
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    ///
    /// // Add a new pair
    /// assert_eq!(map.try_insert(37, "a"), Ok(None));
    /// assert_eq!(map.is_empty(), false);
    ///
    /// // Replace existing pair
    /// map.insert(37, "b");
    /// assert_eq!(map.try_insert(37, "c"), Ok(Some("b")));
    /// assert_eq!(map[&37], "c");
    ///
    /// // Fill remaining capacity
    /// let mut key = 38;
    /// while map.len() < map.capacity() {
    ///     assert!(map.try_insert(key, "filler").is_ok());
    ///     key += 1;
    /// }
    ///
    /// // Full
    /// assert!(map.is_full());
    ///
    /// // Cannot insert new pair
    /// assert_eq!(map.try_insert(key, "out of bounds"), Err(SgError::StackCapacityExceeded));
    ///
    /// // Can still replace existing pair
    /// assert_eq!(map.try_insert(key - 1, "overwrite filler"), Ok(Some("filler")));
    /// ```
    ///
    /// ### Warning
    ///
    /// Unlike other APIs in this crate, the semantics and return type of this API are NOT the same as `BTreeMap`'s nightly [`try_insert`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.try_insert).
    pub fn try_insert(&mut self, key: K, val: V) -> Result<Option<V>, SgError>
    where
        K: Ord,
    {
        self.bst.try_insert(key, val)
    }

    /// Attempt to extend a collection with the contents of an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::iter::FromIterator;
    /// use buggy_scapegoat::{SgMap, SgError};
    ///
    /// let mut a = SgMap::<_, _, 2>::new();
    /// let mut b = SgMap::<_, _, 3>::from_iter([(1, "a"), (2, "b"), (3, "c")]);
    /// let mut c = SgMap::<_, _, 2>::from_iter([(1, "a"), (2, "b")]);
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
    pub fn try_extend<I: ExactSizeIterator + IntoIterator<Item = (K, V)>>(
        &mut self,
        iter: I,
    ) -> Result<(), SgError> {
        self.bst.try_extend(iter)
    }

    /// Attempt conversion from an iterator.
    /// Will fail if iterator length exceeds `u16::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::{SgMap, SgError};
    ///
    /// const CAPACITY_1: usize = 1_000;
    /// let vec: Vec<(usize, usize)> = (0..CAPACITY_1).map(|n|(n, n)).collect();
    /// assert!(SgMap::<usize, usize, CAPACITY_1>::try_from_iter(vec.into_iter()).is_ok());
    ///
    /// const CAPACITY_2: usize = (u16::MAX as usize) + 1;
    /// let vec: Vec<(usize, usize)> = (0..CAPACITY_2).map(|n|(n, n)).collect();
    /// assert_eq!(
    ///     SgMap::<usize, usize, CAPACITY_2>::try_from_iter(vec.into_iter()),
    ///     Err(SgError::MaximumCapacityExceeded)
    /// );
    /// ```
    ///
    /// ### Note
    ///
    /// There is no `TryFromIterator` trait in `core`/`std`.
    pub fn try_from_iter<I: ExactSizeIterator + IntoIterator<Item = (K, V)>>(
        iter: I,
    ) -> Result<Self, SgError> {
        match iter.len() <= SgTree::<K, V, N>::max_capacity() {
            true => Ok(SgMap::from_iter(iter)),
            false => Err(SgError::MaximumCapacityExceeded),
        }
    }

    /// Gets an iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    ///
    /// for (key, value) in map.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    ///
    /// let (first_key, first_value) = map.iter().next().unwrap();
    /// assert_eq!((*first_key, *first_value), (1, "a"));
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V, N> {
        Iter::new(self)
    }

    /// Gets a mutable iterator over the entries of the map, sorted by key.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// // Add 10 to the value if the key isn't "a"
    /// for (key, value) in map.iter_mut() {
    ///     if key != &"a" {
    ///         *value += 10;
    ///     }
    /// }
    ///
    /// let (second_key, second_value) = map.iter().skip(1).next().unwrap();
    /// assert_eq!((*second_key, *second_value), ("b", 12));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, N> {
        IterMut::new(self)
    }

    /// Removes a key from the map, returning the stored key and value if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove_entry(&1), None);
    /// ```
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.remove_entry(key)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
    /// The elements are visited in ascending key order.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map: SgMap<i32, i32, 10> = (0..8).map(|x| (x, x*10)).collect();
    /// // Keep only the elements with even-numbered keys.
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        K: Ord,
        F: FnMut(&K, &mut V) -> bool,
    {
        self.bst.retain(|k, v| f(k, v));
    }

    /// Splits the collection into two at the given key. Returns everything after the given key,
    /// including the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    /// a.insert(17, "d");
    /// a.insert(41, "e");
    ///
    /// let b = a.split_off(&3);
    ///
    /// assert_eq!(a.len(), 2);
    /// assert_eq!(b.len(), 3);
    ///
    /// assert_eq!(a[&1], "a");
    /// assert_eq!(a[&2], "b");
    ///
    /// assert_eq!(b[&3], "c");
    /// assert_eq!(b[&17], "d");
    /// assert_eq!(b[&41], "e");
    /// ```
    pub fn split_off<Q>(&mut self, key: &Q) -> SgMap<K, V, N>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        SgMap {
            bst: self.bst.split_off(key),
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.remove(key)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.get_key_value(key)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.get(key)
    }

    // Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.get_mut(key)
    }

    /// Clears the map, removing all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.bst.clear()
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.bst.contains_key(key)
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bst.is_empty()
    }

    /// Returns `true` if the map's capacity is filled.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 2>::new();
    /// a.insert(1, "a");
    /// assert!(!a.is_full());
    /// a.insert(2, "b");
    /// assert!(a.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.bst.is_full()
    }

    /// Returns a reference to the first key-value pair in the map.
    /// The key in this pair is the minimum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// assert_eq!(map.first_key_value(), None);
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.first_key_value(), Some((&1, &"b")));
    /// ```
    pub fn first_key_value(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        self.bst.first_key_value()
    }

    /// Returns a reference to the first/minium key in the map, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// assert_eq!(map.first_key_value(), None);
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.first_key(), Some(&1));
    /// ```
    pub fn first_key(&self) -> Option<&K>
    where
        K: Ord,
    {
        self.bst.first_key()
    }

    /// Removes and returns the first element in the map.
    /// The key of this element is the minimum key that was in the map.
    ///
    /// # Examples
    ///
    /// Draining elements in ascending order, while keeping a usable map each iteration.
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_first() {
    ///     assert!((&map).into_iter().all(|(k, _v)| *k > key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    pub fn pop_first(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        self.bst.pop_first()
    }

    /// Returns a reference to the last key-value pair in the map.
    /// The key in this pair is the maximum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.last_key_value(), Some((&2, &"a")));
    /// ```
    pub fn last_key_value(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        self.bst.last_key_value()
    }

    /// Returns a reference to the last/maximum key in the map, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "b");
    /// map.insert(2, "a");
    /// assert_eq!(map.last_key(), Some(&2));
    /// ```
    pub fn last_key(&self) -> Option<&K>
    where
        K: Ord,
    {
        self.bst.last_key()
    }

    /// Removes and returns the last element in the map.
    /// The key of this element is the maximum key that was in the map.
    ///
    /// # Examples
    ///
    /// Draining elements in descending order, while keeping a usable map each iteration.
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_last() {
    ///     assert!((&map).into_iter().all(|(k, _v)| *k < key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        self.bst.pop_last()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut a = SgMap::<_, _, 10>::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.bst.len()
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut count = SgMap::<&str, usize, 10>::new();
    ///
    /// // count the number of occurrences of letters in the vec
    /// for x in vec!["a", "b", "a", "c", "a", "b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, N> {
        use crate::tree::node::NodeGetHelper;
        use crate::tree::Idx;

        let ngh: NodeGetHelper<Idx> = self.bst.priv_get(None, &key);
        match ngh.node_idx() {
            Some(node_idx) => Entry::Occupied(OccupiedEntry {
                node_idx,
                table: self,
            }),
            None => Entry::Vacant(VacantEntry { key, table: self }),
        }
    }

    /// Returns the first entry in the map for in-place manipulation.
    /// The key of this entry is the minimum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// if let Some(mut entry) = map.first_entry() {
    ///     if *entry.key() > 0 {
    ///         entry.insert("first");
    ///     }
    /// }
    /// assert_eq!(*map.get(&1).unwrap(), "first");
    /// assert_eq!(*map.get(&2).unwrap(), "b");
    /// ```
    pub fn first_entry(&mut self) -> Option<OccupiedEntry<'_, K, V, N>> {
        if self.is_empty() {
            return None;
        }

        let node_idx = self.bst.min_idx;
        Some(OccupiedEntry {
            node_idx,
            table: self,
        })
    }

    /// Returns the last entry in the map for in-place manipulation.
    /// The key of this entry is the maximum key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let mut map = SgMap::<_, _, 10>::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// if let Some(mut entry) = map.last_entry() {
    ///     if *entry.key() > 0 {
    ///         entry.insert("last");
    ///     }
    /// }
    /// assert_eq!(*map.get(&1).unwrap(), "a");
    /// assert_eq!(*map.get(&2).unwrap(), "last");
    /// ```
    pub fn last_entry(&mut self) -> Option<OccupiedEntry<'_, K, V, N>> {
        if self.is_empty() {
            return None;
        }

        let node_idx = self.bst.max_idx;
        Some(OccupiedEntry {
            node_idx,
            table: self,
        })
    }
}

// Convenience Traits --------------------------------------------------------------------------------------------------

// Debug
impl<K: Default, V: Default, const N: usize> Debug for SgMap<K, V, N>
where
    K: Ord + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.bst.iter()).finish()
    }
}

// From array.
impl<K: Default, V: Default, const N: usize> From<[(K, V); N]> for SgMap<K, V, N>
where
    K: Ord,
{
    /// ```
    /// use buggy_scapegoat::SgMap;
    ///
    /// let map1 = SgMap::from([(1, 2), (3, 4)]);
    /// let map2: SgMap<_, _, 2> = [(1, 2), (3, 4)].into();
    /// assert_eq!(map1, map2);
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
    fn from(arr: [(K, V); N]) -> Self {
        IntoIterator::into_iter(arr).collect()
    }
}

// Indexing
impl<K: Default, V: Default, Q, const N: usize> Index<&Q> for SgMap<K, V, N>
where
    K: Borrow<Q> + Ord,
    Q: Ord + ?Sized,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `SgMap`.
    fn index(&self, key: &Q) -> &Self::Output {
        &self.bst[key]
    }
}

// Construct from iterator.
impl<K: Default, V: Default, const N: usize> FromIterator<(K, V)> for SgMap<K, V, N>
where
    K: Ord,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut sgm = SgMap::new();
        sgm.bst = SgTree::from_iter(iter);
        sgm
    }
}

// Extension from iterator.
impl<K: Default, V: Default, const N: usize> Extend<(K, V)> for SgMap<K, V, N>
where
    K: Ord,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.bst.extend(iter);
    }
}

// Extension from reference iterator.
impl<'a, K: Default, V: Default, const N: usize> Extend<(&'a K, &'a V)> for SgMap<K, V, N>
where
    K: Ord + Copy,
    V: Copy,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

// General Iterators ---------------------------------------------------------------------------------------------------

// Reference iterator
impl<'a, K: Ord + Default, V: Default, const N: usize> IntoIterator for &'a SgMap<K, V, N> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Consuming iterator
impl<K: Ord + Default, V: Default, const N: usize> IntoIterator for SgMap<K, V, N> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
