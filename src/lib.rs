//! # multi-map
//!
//! `MultiMap` is like a `std::collection::HashMap`, but allows you to use either of
//! two different keys to retrieve items.
//!
//! The keys have two distinct types - `K1` and `K2` - which may be the same.
//! Accessing on the primary `K1` key is via the usual `get`, `get_mut` and
//! `remove_alt` methods, while accessing via the secondary `K2` key is via new
//! `get_alt`, `get_mut_alt` and `remove_alt` methods. The value is of type `V`.
//!
//! Internally, two `HashMap`s are created - a main one on `<K1, (K2,
//! V)>` and a second one on `<K2, K1>`. The `(K2, V)` tuple is so
//! that when an item is removed using the `K1` key, the appropriate `K2`
//! value is available so the `K2->K1` map can be removed from the second
//! `MultiMap`, to keep them in sync.
//!
//! Using two `HashMap`s instead of one naturally brings a slight performance
//! and memory penalty. Notably, indexing by `K2` requires two `HashMap` lookups.
//!
//! ```
//! extern crate multi_map;
//! use multi_map::MultiMap;
//!
//! # fn main() {
//! #[derive(Hash,Clone,PartialEq,Eq)]
//! enum ThingIndex {
//!     IndexOne,
//!     IndexTwo,
//!     IndexThree,
//! };
//!
//! let mut map = MultiMap::new();
//! map.insert(1, ThingIndex::IndexOne, "Chicken Fried Steak");
//! map.insert(2, ThingIndex::IndexTwo, "Blueberry Pancakes");
//!
//! assert!(*map.get_alt(&ThingIndex::IndexOne).unwrap() == "Chicken Fried Steak");
//! assert!(*map.get(&2).unwrap() == "Blueberry Pancakes");
//! assert!(map.remove_alt(&ThingIndex::IndexTwo).unwrap() == "Blueberry Pancakes");
//! # }
//! ```

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::Hash;

#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(from = "HashMap<K1, (K2, V)>")
)]
#[derive(Eq)]
pub struct MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    #[cfg_attr(feature = "serde", serde(flatten))]
    value_map: HashMap<K1, (K2, V)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    key_map: HashMap<K2, K1>,
}

impl<K1, K2, V> From<HashMap<K1, (K2, V)>> for MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    fn from(tuple_map: HashMap<K1, (K2, V)>) -> Self {
        let mut m = MultiMap::with_capacity(tuple_map.len());
        for (k1, (k2, v)) in tuple_map {
            m.insert(k1, k2, v);
        }
        m
    }
}

impl<K1, K2, V> MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    /// Creates a new MultiMap. The primary key is of type `K1` and the
    /// secondary key is of type `K2`. The value is of type `V`. This is as
    /// compared to a `std::collections::HashMap` which is typed on just `K` and
    /// `V`.
    ///
    /// Internally, two HashMaps are created - a main one on `<K1, (K2,
    /// V)>` and a second one on `<K2, K1>`. The `(K2, V)` tuple is so
    /// that when an item is removed using the `K1` key, the appropriate `K2`
    /// value is available so the `K2->K1` map can be removed from the second
    /// HashMap, to keep them in sync.
    pub fn new() -> MultiMap<K1, K2, V> {
        MultiMap {
            value_map: HashMap::new(),
            key_map: HashMap::new(),
        }
    }

    /// Creates an empty MultiMap with the specified capacity.
    ///
    /// The multi map will be able to hold at least `capacity` elements without reallocating. If `capacity` is 0, the multi map will not allocate.
    pub fn with_capacity(capacity: usize) -> MultiMap<K1, K2, V> {
        MultiMap {
            value_map: HashMap::with_capacity(capacity),
            key_map: HashMap::with_capacity(capacity),
        }
    }

    /// Insert an item into the MultiMap. You must supply both keys to insert
    /// an item. The keys cannot be modified at a later date, so if you only
    /// have one key at this time, use a placeholder value for the second key
    /// (perhaps `K2` is `Option<...>`) and remove then re-insert when the
    /// second key becomes available.
    pub fn insert(&mut self, key_one: K1, key_two: K2, value: V) {
        self.key_map.insert(key_two.clone(), key_one.clone());
        self.value_map.insert(key_one, (key_two, value));
    }

    /// Obtain a reference to an item in the MultiMap using the primary key,
    /// just like a HashMap.
    pub fn get(&self, key: &K1) -> Option<&V> {
        let mut result = None;
        if let Some(pair) = self.value_map.get(key) {
            result = Some(&pair.1)
        }
        result
    }

    /// Obtain a mutable reference to an item in the MultiMap using the
    /// primary key, just like a HashMap.
    pub fn get_mut(&mut self, key: &K1) -> Option<&mut V> {
        let mut result = None;
        if let Some(pair) = self.value_map.get_mut(key) {
            result = Some(&mut pair.1)
        }
        result
    }

    /// Obtain a reference to an item in the MultiMap using the secondary key.
    /// Ordinary HashMaps can't do this.
    pub fn get_alt(&self, key: &K2) -> Option<&V> {
        let mut result = None;
        if let Some(key_a) = self.key_map.get(key) {
            if let Some(pair) = self.value_map.get(key_a) {
                result = Some(&pair.1)
            }
        }
        result
    }

    /// Obtain a mutable reference to an item in the MultiMap using the
    /// secondary key. Ordinary HashMaps can't do this.
    pub fn get_mut_alt(&mut self, key: &K2) -> Option<&mut V> {
        let mut result = None;
        if let Some(key_a) = self.key_map.get(key) {
            if let Some(pair) = self.value_map.get_mut(key_a) {
                result = Some(&mut pair.1)
            }
        }
        result
    }

    /// Remove an item from the HashMap using the primary key. The value for the
    /// given key is returned (if it exists), just like a HashMap. This removes
    /// an item from the main HashMap, and the second `<K2, K1>` HashMap.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K1: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut result = None;
        if let Some(pair) = self.value_map.remove(key) {
            self.key_map.remove(&pair.0);
            result = Some(pair.1)
        }
        result
    }

    /// Returns true if the map contains a value for the specified key. The key may be any borrowed
    /// form of the map's key type, but Hash and Eq on the borrowed form must match those for the
    /// key type
    ///
    /// ## Example
    /// ```
    /// #[macro_use]
    /// extern crate multi_map;
    /// use multi_map::MultiMap;
    /// # fn main() {
    /// let map = multimap! {
    ///     1, "One" => String::from("Eins"),
    ///     2, "Two" => String::from("Zwei"),
    ///     3, "Three" => String::from("Drei"),
    /// };
    /// assert!(map.contains_key(&1));
    /// assert!(!map.contains_key(&4));
    /// # }
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K1: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.value_map.contains_key(key)
    }

    /// Returns true if the map contains a value for the specified alternative key. The key may be
    /// any borrowed form of the map's key type, but Hash and Eq on the borrowed form must match
    /// those for the key type
    ///
    /// ## Example
    /// ```
    /// #[macro_use]
    /// extern crate multi_map;
    /// use multi_map::MultiMap;
    /// # fn main() {
    /// let map = multimap! {
    ///     1, "One" => String::from("Eins"),
    ///     2, "Two" => String::from("Zwei"),
    ///     3, "Three" => String::from("Drei"),
    /// };
    /// assert!(map.contains_key_alt(&"One"));
    /// assert!(!map.contains_key_alt(&"Four"));
    /// # }
    /// ```
    pub fn contains_key_alt<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K2: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.key_map.contains_key(key)
    }

    /// Remove an item from the HashMap using the secondary key. The value for
    /// the given key is returned (if it exists). Ordinary HashMaps can't do
    /// this. This removes an item from both the main HashMap and the second
    /// `<K2, K1>` HashMap.
    pub fn remove_alt<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K2: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut result = None;
        if let Some(key_a) = self.key_map.remove(key) {
            if let Some(pair) = self.value_map.remove(&key_a) {
                result = Some(pair.1)
            }
        }
        result
    }

    /// Iterate through all the values in the MultiMap in random order.
    /// Note that the values
    /// are `(K2, V)` tuples, not `V`, as you would get with a HashMap.
    pub fn iter(&self) -> Iter<'_, K1, K2, V> {
        Iter {
            base: self.value_map.iter(),
        }
    }
}

impl<K1, K2, V: Eq> PartialEq for MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    fn eq(&self, other: &MultiMap<K1, K2, V>) -> bool {
        self.value_map.eq(&other.value_map)
    }
}

impl<K1, K2, V> fmt::Debug for MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone + Debug,
    K2: Eq + Hash + Clone + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.value_map
                    .iter()
                    .map(|(key_one, &(ref key_two, ref value))| ((key_one, key_two), value)),
            )
            .finish()
    }
}

impl<K1, K2, V> Default for MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    /// Creates an empty `MultiMap<K1, K2, V>`
    #[inline]
    fn default() -> MultiMap<K1, K2, V> {
        MultiMap::new()
    }
}

/// An iterator over the entries of a `MultiMap` like in a `HashMap` but with
/// values of the form (K2, V) instead of V.
///
///
/// This `struct` is created by the [`iter`] method on [`MultiMap`]. See its
/// documentation for more.
///
#[derive(Clone)]
pub struct Iter<'a, K1: 'a, K2: 'a, V: 'a> {
    base: hash_map::Iter<'a, K1, (K2, V)>,
}

/// An owning iterator over the entries of a `MultiMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`MultiMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
pub struct IntoIter<K1, K2, V> {
    base: hash_map::IntoIter<K1, (K2, V)>,
}
// TODO: `HashMap` also implements this, do we need this as well?
// impl<K, V> IntoIter<K, V> {
//     /// Returns a iterator of references over the remaining items.
//     #[inline]
//     pub(super) fn iter(&self) -> Iter<'_, K, V> {
//         Iter { base: self.base.rustc_iter() }
//     }
// }

impl<K1, K2, V> IntoIterator for MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    type Item = (K1, (K2, V));
    type IntoIter = IntoIter<K1, K2, V>;

    /// Creates a consuming iterator, that is, one that moves each key-value
    /// pair out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    ///
    fn into_iter(self) -> IntoIter<K1, K2, V> {
        IntoIter {
            base: self.value_map.into_iter(),
        }
    }
}

impl<'a, K1, K2, V> IntoIterator for &'a MultiMap<K1, K2, V>
where
    K1: Eq + Hash + Clone,
    K2: Eq + Hash + Clone,
{
    type Item = (&'a K1, &'a (K2, V));
    type IntoIter = Iter<'a, K1, K2, V>;

    fn into_iter(self) -> Iter<'a, K1, K2, V> {
        self.iter()
    }
}

impl<'a, K1, K2, V> Iterator for Iter<'a, K1, K2, V> {
    type Item = (&'a K1, &'a (K2, V));

    fn next(&mut self) -> Option<(&'a K1, &'a (K2, V))> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}

impl<K1, K2, V> Iterator for IntoIter<K1, K2, V> {
    type Item = (K1, (K2, V));

    #[inline]
    fn next(&mut self) -> Option<(K1, (K2, V))> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}

#[macro_export]
/// Create a `MultiMap` from a list of key-value tuples
///
/// ## Example
///
/// ```
/// #[macro_use]
/// extern crate multi_map;
/// use multi_map::MultiMap;
///
/// # fn main() {
/// #[derive(Hash,Clone,PartialEq,Eq)]
/// enum ThingIndex {
///     IndexOne,
///     IndexTwo,
///     IndexThree,
/// };
///
/// let map = multimap!{
///     1, ThingIndex::IndexOne => "Chicken Fried Steak",
///     2, ThingIndex::IndexTwo => "Blueberry Pancakes",
/// };
///
/// assert!(*map.get_alt(&ThingIndex::IndexOne).unwrap() == "Chicken Fried Steak");
/// assert!(*map.get(&2).unwrap() == "Blueberry Pancakes");
/// # }
/// ```
macro_rules! multimap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(multimap!(@single $rest)),*]));

    ($($key1:expr, $key2:expr => $value:expr,)+) => { multimap!($($key1, $key2 => $value),+) };
    ($($key1:expr, $key2:expr => $value:expr),*) => {
        {
            let _cap = multimap!(@count $($key1),*);
            let mut _map = MultiMap::with_capacity(_cap);
            $(
                _map.insert($key1, $key2, $value);
            )*
            _map
        }
    };
}

mod test {

    #[test]
    fn big_test() {
        use super::MultiMap;

        let mut map = MultiMap::new();

        map.insert(1, "One", String::from("Ein"));
        map.insert(2, "Two", String::from("Zwei"));
        map.insert(3, "Three", String::from("Drei"));

        assert!(*map.get(&1).unwrap() == String::from("Ein"));
        assert!(*map.get(&2).unwrap() == String::from("Zwei"));
        assert!(*map.get(&3).unwrap() == String::from("Drei"));
        assert!(map.contains_key(&1));
        assert!(!map.contains_key(&4));
        assert!(map.contains_key_alt(&"One"));
        assert!(!map.contains_key_alt(&"Four"));

        map.get_mut_alt(&"One").unwrap().push_str("s");

        assert!(*map.get_alt(&"One").unwrap() == String::from("Eins"));
        assert!(*map.get_alt(&"Two").unwrap() == String::from("Zwei"));
        assert!(*map.get_alt(&"Three").unwrap() == String::from("Drei"));

        map.remove(&3);

        assert!(*map.get_alt(&"One").unwrap() == String::from("Eins"));
        assert!(*map.get_alt(&"Two").unwrap() == String::from("Zwei"));
        assert!(map.get_alt(&"Three") == None);
        assert!(map.get(&3) == None);

        assert!(map.remove_alt(&"Three") == None);
        assert!(*map.remove_alt(&"One").unwrap() == String::from("Eins"));

        map.get_mut(&2).unwrap().push_str("!");

        assert!(map.get(&1) == None);
        assert!(*map.get(&2).unwrap() == String::from("Zwei!"));
        assert!(map.get_alt(&"Three") == None);
        assert!(map.get(&3) == None);
    }
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct MultiCount<'a>(i32, &'a str, &'a str);
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct MultiCountOwned(i32, String, String);

    #[test]
    fn into_iter_test() {
        use super::MultiMap;
        let mut map = MultiMap::new();

        map.insert(1, "One", String::from("Eins"));
        map.insert(2, "Two", String::from("Zwei"));
        map.insert(3, "Three", String::from("Drei"));

        let mut vec_borrow = Vec::new();
        for (k1, (k2, v)) in &map {
            vec_borrow.push(MultiCount(*k1, *k2, v));
        }
        vec_borrow.sort();
        assert_eq!(
            vec_borrow,
            vec!(
                MultiCount(1, "One", "Eins"),
                MultiCount(2, "Two", "Zwei"),
                MultiCount(3, "Three", "Drei")
            )
        );

        let mut vec_owned = Vec::new();
        for (k1, (k2, v)) in map {
            vec_owned.push(MultiCountOwned(k1, String::from(k2), v));
        }
        vec_owned.sort();
        assert_eq!(
            vec_owned,
            vec!(
                MultiCountOwned(1, String::from("One"), String::from("Eins")),
                MultiCountOwned(2, String::from("Two"), String::from("Zwei")),
                MultiCountOwned(3, String::from("Three"), String::from("Drei"))
            )
        )
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_test() {
        use super::MultiMap;
        let mut map = MultiMap::new();

        map.insert(1, "One", String::from("Eins"));
        map.insert(2, "Two", String::from("Zwei"));
        map.insert(3, "Three", String::from("Drei"));
        let serialized = serde_json::to_string(&map).unwrap();

        let deserialized: MultiMap<i32, &str, String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(*deserialized.get(&1).unwrap(), String::from("Eins"));
        assert_eq!(*deserialized.get_alt(&"One").unwrap(), String::from("Eins"));

        assert_eq!(*deserialized.get(&2).unwrap(), String::from("Zwei"));
        assert_eq!(*deserialized.get_alt(&"Two").unwrap(), String::from("Zwei"));

        assert_eq!(*deserialized.get(&3).unwrap(), String::from("Drei"));
        assert_eq!(
            *deserialized.get_alt(&"Three").unwrap(),
            String::from("Drei")
        );

        assert_eq!(deserialized.get(&4), None);
        assert_eq!(deserialized.get_alt(&"Four"), None);
    }

    #[test]
    fn macro_test() {
        use super::MultiMap;

        let map: MultiMap<i32, &str, String> = MultiMap::new();

        assert_eq!(map, multimap! {});

        let mut map = MultiMap::new();
        map.insert(1, "One", String::from("Eins"));

        assert_eq!(
            map,
            multimap! {
                1, "One" => String::from("Eins"),
            }
        );

        assert_eq!(
            map,
            multimap! {
                1, "One" => String::from("Eins")
            }
        );

        let mut map = MultiMap::new();
        map.insert(1, "One", String::from("Eins"));
        map.insert(2, "Two", String::from("Zwei"));
        map.insert(3, "Three", String::from("Drei"));

        assert_eq!(
            map,
            multimap! {
                1, "One" => String::from("Eins"),
                2, "Two" => String::from("Zwei"),
                3, "Three" => String::from("Drei"),
            }
        );

        assert_eq!(
            map,
            multimap! {
                1, "One" => String::from("Eins"),
                2, "Two" => String::from("Zwei"),
                3, "Three" => String::from("Drei")
            }
        );
    }
}
