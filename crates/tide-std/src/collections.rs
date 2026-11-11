//! Rust's collections with capability-gated randomized hashing (03 GLOBAL-5).
//! HashMap/HashSet keep the pinned toolchain's full stable Rust API, including
//! its signatures, traits, operators, and entry/iterator types.
//! Their fixed default key gives repeatable, unspecified iteration order for the
//! same mutation history on the pinned toolchain and target. Use `from_rand` for
//! untrusted keys. `std::hash::RandomState` remains Rust's distinct type.

pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use std::collections::{
    BTreeMap, BTreeSet, BinaryHeap, LinkedList, TryReserveError, VecDeque, binary_heap, btree_map,
    btree_set, linked_list, vec_deque,
};

pub mod hash_map {
    use std::borrow::Borrow;
    pub use std::collections::hash_map::{
        Drain, Entry, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys,
        OccupiedEntry, VacantEntry, Values, ValuesMut,
    };
    use std::hash::{BuildHasher, Hash};

    /// The default collection builder (03 GLOBAL-5). `default()` uses a fixed key;
    /// `from_rand()` uses OS-seeded keys and requires randomness authority.
    /// Simulated randomness is unavailable; `from_rand` rejects it until its backend exists.
    #[derive(Clone)]
    pub struct RandomState(std::hash::DefaultHasher);
    impl std::fmt::Debug for RandomState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // DefaultHasher's Debug reveals its key; keep the builder's key private.
            f.debug_struct("RandomState").finish_non_exhaustive()
        }
    }
    impl RandomState {
        pub fn from_rand(rand: &crate::rand::Rand) -> RandomState {
            if let Some(tasks) = &rand.0 {
                tasks.unmodeled("Rand");
            }
            crate::task::outside_simulation();
            RandomState(std::hash::RandomState::new().build_hasher())
        }
    }
    impl Default for RandomState {
        fn default() -> Self {
            // The pinned DefaultHasher is SipHash-1-3 with key (0, 0).
            Self(std::hash::DefaultHasher::new())
        }
    }
    impl BuildHasher for RandomState {
        type Hasher = std::hash::DefaultHasher;
        fn build_hasher(&self) -> Self::Hasher {
            self.0.clone()
        }
    }

    /// Wraps Rust's map so that variance, drop check, and auto traits stay Rust's.
    pub struct HashMap<K, V, S = RandomState> {
        base: std::collections::HashMap<K, V, S>,
    }
    impl<K, V> HashMap<K, V> {
        pub fn new() -> HashMap<K, V> {
            Self::default()
        }
        pub fn with_capacity(capacity: usize) -> HashMap<K, V> {
            Self {
                base: std::collections::HashMap::with_capacity_and_hasher(
                    capacity,
                    RandomState::default(),
                ),
            }
        }
    }
    impl<K, V, S> HashMap<K, V, S> {
        /// Builds the value, so that a `static` using it evaluates, as in Rust.
        pub const fn with_hasher(hasher: S) -> HashMap<K, V, S> {
            HashMap {
                base: std::collections::HashMap::with_hasher(hasher),
            }
        }
        pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> HashMap<K, V, S> {
            HashMap {
                base: std::collections::HashMap::with_capacity_and_hasher(capacity, hasher),
            }
        }
        pub fn capacity(&self) -> usize {
            self.base.capacity()
        }
        pub fn hasher(&self) -> &S {
            self.base.hasher()
        }
        pub fn len(&self) -> usize {
            self.base.len()
        }
        pub fn is_empty(&self) -> bool {
            self.base.is_empty()
        }
        pub fn iter(&self) -> Iter<'_, K, V> {
            self.base.iter()
        }
        pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
            self.base.iter_mut()
        }
        pub fn keys(&self) -> Keys<'_, K, V> {
            self.base.keys()
        }
        pub fn values(&self) -> Values<'_, K, V> {
            self.base.values()
        }
        pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
            self.base.values_mut()
        }
        pub fn into_keys(self) -> IntoKeys<K, V> {
            self.base.into_keys()
        }
        pub fn into_values(self) -> IntoValues<K, V> {
            self.base.into_values()
        }
        pub fn clear(&mut self) {
            self.base.clear()
        }
        pub fn drain(&mut self) -> Drain<'_, K, V> {
            self.base.drain()
        }
        pub fn extract_if<F>(&mut self, pred: F) -> ExtractIf<'_, K, V, F>
        where
            F: FnMut(&K, &mut V) -> bool,
        {
            self.base.extract_if(pred)
        }
        pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, f: F) {
            self.base.retain(f)
        }
    }
    impl<K: Eq + Hash, V, S: BuildHasher> HashMap<K, V, S> {
        pub fn reserve(&mut self, additional: usize) {
            self.base.reserve(additional)
        }
        pub fn try_reserve(&mut self, additional: usize) -> Result<(), super::TryReserveError> {
            self.base.try_reserve(additional)
        }
        pub fn shrink_to_fit(&mut self) {
            self.base.shrink_to_fit()
        }
        pub fn shrink_to(&mut self, min_capacity: usize) {
            self.base.shrink_to(min_capacity)
        }
        pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
            self.base.entry(key)
        }
        pub fn get_key_value<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<(&K, &V)>
        where
            K: Borrow<Q>,
        {
            self.base.get_key_value(k)
        }
        pub fn get<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
        {
            self.base.get(k)
        }
        pub fn get_mut<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<&mut V>
        where
            K: Borrow<Q>,
        {
            self.base.get_mut(k)
        }
        /// Panics if any keys refer to the same value.
        pub fn get_disjoint_mut<Q: ?Sized, const N: usize>(
            &mut self,
            ks: [&Q; N],
        ) -> [Option<&'_ mut V>; N]
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
        {
            self.base.get_disjoint_mut(ks)
        }
        /// Looks up multiple mutable values without checking for overlap.
        ///
        /// # Safety
        /// The keys must not refer to overlapping values, even if the results are unused.
        pub unsafe fn get_disjoint_unchecked_mut<Q: ?Sized, const N: usize>(
            &mut self,
            ks: [&Q; N],
        ) -> [Option<&'_ mut V>; N]
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
        {
            unsafe { self.base.get_disjoint_unchecked_mut(ks) }
        }
        pub fn contains_key<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> bool
        where
            K: Borrow<Q>,
        {
            self.base.contains_key(k)
        }
        pub fn insert(&mut self, k: K, v: V) -> Option<V> {
            self.base.insert(k, v)
        }
        pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, k: &Q) -> Option<V>
        where
            K: Borrow<Q>,
        {
            self.base.remove(k)
        }
        pub fn remove_entry<Q: ?Sized>(&mut self, k: &Q) -> Option<(K, V)>
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
        {
            self.base.remove_entry(k)
        }
    }
    impl<K: Clone, V: Clone, S: Clone> Clone for HashMap<K, V, S> {
        fn clone(&self) -> Self {
            Self {
                base: self.base.clone(),
            }
        }
        fn clone_from(&mut self, source: &Self) {
            self.base.clone_from(&source.base)
        }
    }
    impl<K: std::fmt::Debug, V: std::fmt::Debug, S> std::fmt::Debug for HashMap<K, V, S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.base.fmt(f)
        }
    }
    impl<K: Eq + Hash, V: PartialEq, S: BuildHasher> PartialEq for HashMap<K, V, S> {
        fn eq(&self, other: &Self) -> bool {
            self.base.eq(&other.base)
        }
    }
    impl<K: Eq + Hash, V: Eq, S: BuildHasher> Eq for HashMap<K, V, S> {}
    impl<K, V, S: Default> Default for HashMap<K, V, S> {
        fn default() -> Self {
            Self::with_hasher(S::default())
        }
    }
    impl<K: Eq + Hash, V, S: BuildHasher + Default> FromIterator<(K, V)> for HashMap<K, V, S> {
        fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
            Self {
                base: iter.into_iter().collect(),
            }
        }
    }
    /// A fixed key, as `new()`.
    impl<K: Eq + Hash, V, const N: usize> From<[(K, V); N]> for HashMap<K, V> {
        fn from(arr: [(K, V); N]) -> Self {
            arr.into_iter().collect()
        }
    }
    impl<K: Eq + Hash, V, S: BuildHasher> Extend<(K, V)> for HashMap<K, V, S> {
        fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
            self.base.extend(iter)
        }
    }
    impl<'a, K: Eq + Hash + Copy, V: Copy, S: BuildHasher> Extend<(&'a K, &'a V)> for HashMap<K, V, S> {
        fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
            self.base.extend(iter)
        }
    }
    impl<K, V, S> IntoIterator for HashMap<K, V, S> {
        type Item = (K, V);
        type IntoIter = IntoIter<K, V>;
        fn into_iter(self) -> IntoIter<K, V> {
            self.base.into_iter()
        }
    }
    impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
        type Item = (&'a K, &'a V);
        type IntoIter = Iter<'a, K, V>;
        fn into_iter(self) -> Iter<'a, K, V> {
            (&self.base).into_iter()
        }
    }
    impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
        type Item = (&'a K, &'a mut V);
        type IntoIter = IterMut<'a, K, V>;
        fn into_iter(self) -> IterMut<'a, K, V> {
            (&mut self.base).into_iter()
        }
    }
    impl<K, Q: ?Sized, V, S> std::ops::Index<&Q> for HashMap<K, V, S>
    where
        K: Eq + Hash + Borrow<Q>,
        Q: Eq + Hash,
        S: BuildHasher,
    {
        type Output = V;
        fn index(&self, key: &Q) -> &V {
            &self.base[key]
        }
    }
}

pub mod hash_set {
    use super::hash_map::RandomState;
    use std::borrow::Borrow;
    pub use std::collections::hash_set::{
        Difference, Drain, ExtractIf, Intersection, IntoIter, Iter, SymmetricDifference, Union,
    };
    use std::hash::{BuildHasher, Hash};

    /// Wraps Rust's set so that variance, drop check, and auto traits stay Rust's.
    pub struct HashSet<T, S = RandomState> {
        base: std::collections::HashSet<T, S>,
    }
    impl<T> HashSet<T> {
        pub fn new() -> HashSet<T> {
            Self::default()
        }
        pub fn with_capacity(capacity: usize) -> HashSet<T> {
            Self {
                base: std::collections::HashSet::with_capacity_and_hasher(
                    capacity,
                    RandomState::default(),
                ),
            }
        }
    }
    impl<T, S> HashSet<T, S> {
        /// Builds the value, so that a `static` using it evaluates, as in Rust.
        pub const fn with_hasher(hasher: S) -> HashSet<T, S> {
            HashSet {
                base: std::collections::HashSet::with_hasher(hasher),
            }
        }
        pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> HashSet<T, S> {
            HashSet {
                base: std::collections::HashSet::with_capacity_and_hasher(capacity, hasher),
            }
        }
        pub fn capacity(&self) -> usize {
            self.base.capacity()
        }
        pub fn hasher(&self) -> &S {
            self.base.hasher()
        }
        pub fn len(&self) -> usize {
            self.base.len()
        }
        pub fn is_empty(&self) -> bool {
            self.base.is_empty()
        }
        pub fn iter(&self) -> Iter<'_, T> {
            self.base.iter()
        }
        pub fn clear(&mut self) {
            self.base.clear()
        }
        pub fn drain(&mut self) -> Drain<'_, T> {
            self.base.drain()
        }
        pub fn extract_if<F>(&mut self, pred: F) -> ExtractIf<'_, T, F>
        where
            F: FnMut(&T) -> bool,
        {
            self.base.extract_if(pred)
        }
        pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
            self.base.retain(f)
        }
    }
    impl<T: Eq + Hash, S: BuildHasher> HashSet<T, S> {
        pub fn reserve(&mut self, additional: usize) {
            self.base.reserve(additional)
        }
        pub fn try_reserve(&mut self, additional: usize) -> Result<(), super::TryReserveError> {
            self.base.try_reserve(additional)
        }
        pub fn shrink_to_fit(&mut self) {
            self.base.shrink_to_fit()
        }
        pub fn shrink_to(&mut self, min_capacity: usize) {
            self.base.shrink_to(min_capacity)
        }
        pub fn difference<'a>(&'a self, other: &'a HashSet<T, S>) -> Difference<'a, T, S> {
            self.base.difference(&other.base)
        }
        pub fn symmetric_difference<'a>(
            &'a self,
            other: &'a HashSet<T, S>,
        ) -> SymmetricDifference<'a, T, S> {
            self.base.symmetric_difference(&other.base)
        }
        pub fn intersection<'a>(&'a self, other: &'a HashSet<T, S>) -> Intersection<'a, T, S> {
            self.base.intersection(&other.base)
        }
        pub fn union<'a>(&'a self, other: &'a HashSet<T, S>) -> Union<'a, T, S> {
            self.base.union(&other.base)
        }
        pub fn is_disjoint(&self, other: &HashSet<T, S>) -> bool {
            self.base.is_disjoint(&other.base)
        }
        pub fn is_subset(&self, other: &HashSet<T, S>) -> bool {
            self.base.is_subset(&other.base)
        }
        pub fn is_superset(&self, other: &HashSet<T, S>) -> bool {
            self.base.is_superset(&other.base)
        }
        pub fn insert(&mut self, value: T) -> bool {
            self.base.insert(value)
        }
        pub fn contains<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> bool
        where
            T: Borrow<Q>,
        {
            self.base.contains(value)
        }
        pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
        where
            T: Borrow<Q>,
            Q: Hash + Eq,
        {
            self.base.get(value)
        }
        pub fn replace(&mut self, value: T) -> Option<T> {
            self.base.replace(value)
        }
        pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool
        where
            T: Borrow<Q>,
        {
            self.base.remove(value)
        }
        pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
        where
            T: Borrow<Q>,
            Q: Hash + Eq,
        {
            self.base.take(value)
        }
    }
    impl<T: Clone, S: Clone> Clone for HashSet<T, S> {
        fn clone(&self) -> Self {
            Self {
                base: self.base.clone(),
            }
        }
        fn clone_from(&mut self, other: &Self) {
            self.base.clone_from(&other.base)
        }
    }
    impl<T: std::fmt::Debug, S> std::fmt::Debug for HashSet<T, S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.base.fmt(f)
        }
    }
    impl<T: Eq + Hash, S: BuildHasher> PartialEq for HashSet<T, S> {
        fn eq(&self, other: &Self) -> bool {
            self.base.eq(&other.base)
        }
    }
    impl<T: Eq + Hash, S: BuildHasher> Eq for HashSet<T, S> {}
    impl<T, S: Default> Default for HashSet<T, S> {
        fn default() -> Self {
            Self::with_hasher(S::default())
        }
    }
    impl<T: Eq + Hash, S: BuildHasher + Default> FromIterator<T> for HashSet<T, S> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            Self {
                base: iter.into_iter().collect(),
            }
        }
    }
    /// A fixed key, as `new()`.
    impl<T: Eq + Hash, const N: usize> From<[T; N]> for HashSet<T> {
        fn from(arr: [T; N]) -> Self {
            arr.into_iter().collect()
        }
    }
    impl<T: Eq + Hash, S: BuildHasher> Extend<T> for HashSet<T, S> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            self.base.extend(iter)
        }
    }
    impl<'a, T: 'a + Eq + Hash + Copy, S: BuildHasher> Extend<&'a T> for HashSet<T, S> {
        fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
            self.base.extend(iter)
        }
    }
    impl<T, S> IntoIterator for HashSet<T, S> {
        type Item = T;
        type IntoIter = IntoIter<T>;
        fn into_iter(self) -> IntoIter<T> {
            self.base.into_iter()
        }
    }
    impl<'a, T, S> IntoIterator for &'a HashSet<T, S> {
        type Item = &'a T;
        type IntoIter = Iter<'a, T>;
        fn into_iter(self) -> Iter<'a, T> {
            (&self.base).into_iter()
        }
    }
    impl<T: Eq + Hash + Clone, S: BuildHasher + Default> std::ops::BitOr<&HashSet<T, S>>
        for &HashSet<T, S>
    {
        type Output = HashSet<T, S>;
        fn bitor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
            HashSet {
                base: &self.base | &rhs.base,
            }
        }
    }
    impl<T: Eq + Hash + Clone, S: BuildHasher + Default> std::ops::BitAnd<&HashSet<T, S>>
        for &HashSet<T, S>
    {
        type Output = HashSet<T, S>;
        fn bitand(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
            HashSet {
                base: &self.base & &rhs.base,
            }
        }
    }
    impl<T: Eq + Hash + Clone, S: BuildHasher + Default> std::ops::BitXor<&HashSet<T, S>>
        for &HashSet<T, S>
    {
        type Output = HashSet<T, S>;
        fn bitxor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
            HashSet {
                base: &self.base ^ &rhs.base,
            }
        }
    }
    impl<T: Eq + Hash + Clone, S: BuildHasher + Default> std::ops::Sub<&HashSet<T, S>>
        for &HashSet<T, S>
    {
        type Output = HashSet<T, S>;
        fn sub(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
            HashSet {
                base: &self.base - &rhs.base,
            }
        }
    }
}
