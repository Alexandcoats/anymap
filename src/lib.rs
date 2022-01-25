//! This crate provides a safe and convenient store for one value of each type.
//!
//! Your starting point is [`Map`]. It has an example.

#![warn(missing_docs, unused_results)]

#![cfg_attr(not(feature = "std"), no_std)]

use core::any::{Any, TypeId};
use core::convert::TryInto;
use core::hash::{Hasher, BuildHasherDefault};
use core::marker::PhantomData;

#[cfg(not(any(feature = "std", feature = "hashbrown")))]
compile_error!("anymap: you must enable the 'std' feature or the 'hashbrown' feature");

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use any::{UncheckedAnyExt, IntoBox};
pub use any::CloneAny;

#[cfg(all(feature = "std", not(feature = "hashbrown")))]
/// A re-export of [`std::collections::hash_map`] for raw access.
///
/// If the `hashbrown` feature gets enabled, this will become an export of `hashbrown::hash_map`.
///
/// As with [`RawMap`][crate::RawMap], this is exposed for compatibility reasons, since features
/// are supposed to be additive. This *is* imperfect, since the two modules are incompatible in a
/// few places (e.g. hashbrown’s entry types have an extra generic parameter), but it’s close, and
/// much too useful to give up the whole concept.
pub use std::collections::hash_map as raw_hash_map;

#[cfg(feature = "hashbrown")]
/// A re-export of [`hashbrown::hash_map`] for raw access.
///
/// If the `hashbrown` feature was disabled, this would become an export of
/// `std::collections::hash_map`.
///
/// As with [`RawMap`][crate::RawMap], this is exposed for compatibility reasons, since features
/// are supposed to be additive. This *is* imperfect, since the two modules are incompatible in a
/// few places (e.g. hashbrown’s entry types have an extra generic parameter), but it’s close, and
/// much too useful to give up the whole concept.
pub use hashbrown::hash_map as raw_hash_map;

use self::raw_hash_map::HashMap;

macro_rules! impl_common_methods {
    (
        field: $t:ident.$field:ident;
        new() => $new:expr;
        with_capacity($with_capacity_arg:ident) => $with_capacity:expr;
    ) => {
        impl<A: ?Sized + UncheckedAnyExt> $t<A> {
            /// Create an empty collection.
            #[inline]
            pub fn new() -> $t<A> {
                $t {
                    $field: $new,
                }
            }

            /// Creates an empty collection with the given initial capacity.
            #[inline]
            pub fn with_capacity($with_capacity_arg: usize) -> $t<A> {
                $t {
                    $field: $with_capacity,
                }
            }

            /// Returns the number of elements the collection can hold without reallocating.
            #[inline]
            pub fn capacity(&self) -> usize {
                self.$field.capacity()
            }

            /// Reserves capacity for at least `additional` more elements to be inserted
            /// in the collection. The collection may reserve more space to avoid
            /// frequent reallocations.
            ///
            /// # Panics
            ///
            /// Panics if the new allocation size overflows `usize`.
            #[inline]
            pub fn reserve(&mut self, additional: usize) {
                self.$field.reserve(additional)
            }

            /// Shrinks the capacity of the collection as much as possible. It will drop
            /// down as much as possible while maintaining the internal rules
            /// and possibly leaving some space in accordance with the resize policy.
            #[inline]
            pub fn shrink_to_fit(&mut self) {
                self.$field.shrink_to_fit()
            }

            // Additional stable methods (as of 1.60.0-nightly) that could be added:
            // try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>    (1.57.0)
            // shrink_to(&mut self, min_capacity: usize)                                   (1.56.0)

            /// Returns the number of items in the collection.
            #[inline]
            pub fn len(&self) -> usize {
                self.$field.len()
            }

            /// Returns true if there are no items in the collection.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.$field.is_empty()
            }

            /// Removes all items from the collection. Keeps the allocated memory for reuse.
            #[inline]
            pub fn clear(&mut self) {
                self.$field.clear()
            }
        }

        impl<A: ?Sized + UncheckedAnyExt> Default for $t<A> {
            #[inline]
            fn default() -> $t<A> {
                $t::new()
            }
        }
    }
}

mod any;

/// Raw access to the underlying `HashMap`.
///
/// This is a public type alias because the underlying `HashMap` could be
/// `std::collections::HashMap` or `hashbrown::HashMap`, depending on the crate features enabled.
/// For that reason, you should refer to this type as `anymap::RawMap` rather than
/// `std::collections::HashMap` to avoid breakage if something else in your crate tree enables
/// hashbrown.
///
/// See also [`raw_hash_map`], an export of the corresponding `hash_map` module.
pub type RawMap<A> = HashMap<TypeId, Box<A>, BuildHasherDefault<TypeIdHasher>>;

/// A collection containing zero or one values for any given type and allowing convenient,
/// type-safe access to those values.
///
/// The type parameter `A` allows you to use a different value type; normally you will want it to
/// be `core::any::Any` (also known as `std::any::Any`), but there are other choices:
///
/// - If you want the entire map to be cloneable, use `CloneAny` instead of `Any`; with that, you
///   can only add types that implement `Clone` to the map.
/// - You can add on `+ Send` or `+ Send + Sync` (e.g. `Map<dyn Any + Send>`) to add those auto
///   traits.
///
/// Cumulatively, there are thus six forms of map:
///
/// - <code>[Map]&lt;dyn [core::any::Any]&gt;</code>, also spelled [`AnyMap`] for convenience.
/// - <code>[Map]&lt;dyn [core::any::Any] + Send&gt;</code>
/// - <code>[Map]&lt;dyn [core::any::Any] + Send + Sync&gt;</code>
/// - <code>[Map]&lt;dyn [CloneAny]&gt;</code>
/// - <code>[Map]&lt;dyn [CloneAny] + Send&gt;</code>
/// - <code>[Map]&lt;dyn [CloneAny] + Send + Sync&gt;</code>
///
/// ## Example
///
/// (Here using the [`AnyMap`] convenience alias; the first line could use
/// <code>[anymap::Map][Map]::&lt;[core::any::Any]&gt;::new()</code> instead if desired.)
///
/// ```rust
/// let mut data = anymap::AnyMap::new();
/// assert_eq!(data.get(), None::<&i32>);
/// data.insert(42i32);
/// assert_eq!(data.get(), Some(&42i32));
/// data.remove::<i32>();
/// assert_eq!(data.get::<i32>(), None);
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Foo {
///     str: String,
/// }
///
/// assert_eq!(data.get::<Foo>(), None);
/// data.insert(Foo { str: format!("foo") });
/// assert_eq!(data.get(), Some(&Foo { str: format!("foo") }));
/// data.get_mut::<Foo>().map(|foo| foo.str.push('t'));
/// assert_eq!(&*data.get::<Foo>().unwrap().str, "foot");
/// ```
///
/// Values containing non-static references are not permitted.
#[derive(Debug)]
pub struct Map<A: ?Sized + UncheckedAnyExt = dyn Any> {
    raw: RawMap<A>,
}

// #[derive(Clone)] would want A to implement Clone, but in reality it’s only Box<A> that can.
impl<A: ?Sized + UncheckedAnyExt> Clone for Map<A> where Box<A>: Clone {
    #[inline]
    fn clone(&self) -> Map<A> {
        Map {
            raw: self.raw.clone(),
        }
    }
}

/// The most common type of `Map`: just using `Any`; <code>[Map]&lt;dyn [Any]&gt;</code>.
///
/// Why is this a separate type alias rather than a default value for `Map<A>`? `Map::new()`
/// doesn’t seem to be happy to infer that it should go with the default value.
/// It’s a bit sad, really. Ah well, I guess this approach will do.
pub type AnyMap = Map<dyn Any>;

impl_common_methods! {
    field: Map.raw;
    new() => RawMap::with_hasher(Default::default());
    with_capacity(capacity) => RawMap::with_capacity_and_hasher(capacity, Default::default());
}

impl<A: ?Sized + UncheckedAnyExt> Map<A> {
    /// Returns a reference to the value stored in the collection for the type `T`, if it exists.
    #[inline]
    pub fn get<T: IntoBox<A>>(&self) -> Option<&T> {
        self.raw.get(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_ref_unchecked::<T>() })
    }

    /// Returns a mutable reference to the value stored in the collection for the type `T`,
    /// if it exists.
    #[inline]
    pub fn get_mut<T: IntoBox<A>>(&mut self) -> Option<&mut T> {
        self.raw.get_mut(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_mut_unchecked::<T>() })
    }

    /// Sets the value stored in the collection for the type `T`.
    /// If the collection already had a value of type `T`, that value is returned.
    /// Otherwise, `None` is returned.
    #[inline]
    pub fn insert<T: IntoBox<A>>(&mut self, value: T) -> Option<T> {
        unsafe {
            self.raw.insert(TypeId::of::<T>(), value.into_box())
                .map(|any| *any.downcast_unchecked::<T>())
        }
    }

    // rustc 1.60.0-nightly has another method try_insert that would be nice to add when stable.

    /// Removes the `T` value from the collection,
    /// returning it if there was one or `None` if there was not.
    #[inline]
    pub fn remove<T: IntoBox<A>>(&mut self) -> Option<T> {
        self.raw.remove(&TypeId::of::<T>())
            .map(|any| *unsafe { any.downcast_unchecked::<T>() })
    }

    /// Returns true if the collection contains a value of type `T`.
    #[inline]
    pub fn contains<T: IntoBox<A>>(&self) -> bool {
        self.raw.contains_key(&TypeId::of::<T>())
    }

    /// Gets the entry for the given type in the collection for in-place manipulation
    #[inline]
    pub fn entry<T: IntoBox<A>>(&mut self) -> Entry<A, T> {
        match self.raw.entry(TypeId::of::<T>()) {
            raw_hash_map::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                inner: e,
                type_: PhantomData,
            }),
            raw_hash_map::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                inner: e,
                type_: PhantomData,
            }),
        }
    }

    /// Get access to the raw hash map that backs this.
    ///
    /// This will seldom be useful, but it’s conceivable that you could wish to iterate over all
    /// the items in the collection, and this lets you do that.
    ///
    /// To improve compatibility with Cargo features, interact with this map through the names
    /// [`anymap::RawMap`][RawMap] and [`anymap::raw_hash_map`][raw_hash_map], rather than through
    /// `std::collections::{HashMap, hash_map}` or `hashbrown::{HashMap, hash_map}`, for anything
    /// beyond self methods. Otherwise, if you use std and another crate in the tree enables
    /// hashbrown, your code will break.
    #[inline]
    pub fn as_raw(&self) -> &RawMap<A> {
        &self.raw
    }

    /// Get mutable access to the raw hash map that backs this.
    ///
    /// This will seldom be useful, but it’s conceivable that you could wish to iterate over all
    /// the items in the collection mutably, or drain or something, or *possibly* even batch
    /// insert, and this lets you do that.
    ///
    /// To improve compatibility with Cargo features, interact with this map through the names
    /// [`anymap::RawMap`][RawMap] and [`anymap::raw_hash_map`][raw_hash_map], rather than through
    /// `std::collections::{HashMap, hash_map}` or `hashbrown::{HashMap, hash_map}`, for anything
    /// beyond self methods. Otherwise, if you use std and another crate in the tree enables
    /// hashbrown, your code will break.
    ///
    /// # Safety
    ///
    /// If you insert any values to the raw map, the key (a `TypeId`) must match the value’s type,
    /// or *undefined behaviour* will occur when you access those values.
    ///
    /// (*Removing* entries is perfectly safe.)
    #[inline]
    pub unsafe fn as_raw_mut(&mut self) -> &mut RawMap<A> {
        &mut self.raw
    }

    /// Convert this into the raw hash map that backs this.
    ///
    /// This will seldom be useful, but it’s conceivable that you could wish to consume all the
    /// items in the collection and do *something* with some or all of them, and this lets you do
    /// that, without the `unsafe` that `.as_raw_mut().drain()` would require.
    ///
    /// To improve compatibility with Cargo features, interact with this map through the names
    /// [`anymap::RawMap`][RawMap] and [`anymap::raw_hash_map`][raw_hash_map], rather than through
    /// `std::collections::{HashMap, hash_map}` or `hashbrown::{HashMap, hash_map}`, for anything
    /// beyond self methods. Otherwise, if you use std and another crate in the tree enables
    /// hashbrown, your code will break.
    #[inline]
    pub fn into_raw(self) -> RawMap<A> {
        self.raw
    }

    /// Construct a map from a collection of raw values.
    ///
    /// You know what? I can’t immediately think of any legitimate use for this, especially because
    /// of the requirement of the `BuildHasherDefault<TypeIdHasher>` generic in the map.
    ///
    /// Perhaps this will be most practical as `unsafe { Map::from_raw(iter.collect()) }`, iter
    /// being an iterator over `(TypeId, Box<A>)` pairs. Eh, this method provides symmetry with
    /// `into_raw`, so I don’t care if literally no one ever uses it. I’m not even going to write a
    /// test for it, it’s so trivial.
    ///
    /// To improve compatibility with Cargo features, interact with this map through the names
    /// [`anymap::RawMap`][RawMap] and [`anymap::raw_hash_map`][raw_hash_map], rather than through
    /// `std::collections::{HashMap, hash_map}` or `hashbrown::{HashMap, hash_map}`, for anything
    /// beyond self methods. Otherwise, if you use std and another crate in the tree enables
    /// hashbrown, your code will break.
    ///
    /// # Safety
    ///
    /// For all entries in the raw map, the key (a `TypeId`) must match the value’s type,
    /// or *undefined behaviour* will occur when you access that entry.
    #[inline]
    pub unsafe fn from_raw(raw: RawMap<A>) -> Map<A> {
        Self { raw }
    }
}

impl<A: ?Sized + UncheckedAnyExt> Extend<Box<A>> for Map<A> {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<A>>>(&mut self, iter: T) {
        for item in iter {
            let _ = self.raw.insert(item.type_id(), item);
        }
    }
}

/// A view into a single occupied location in an `Map`.
pub struct OccupiedEntry<'a, A: ?Sized + UncheckedAnyExt, V: 'a> {
    #[cfg(all(feature = "std", not(feature = "hashbrown")))]
    inner: raw_hash_map::OccupiedEntry<'a, TypeId, Box<A>>,
    #[cfg(feature = "hashbrown")]
    inner: raw_hash_map::OccupiedEntry<'a, TypeId, Box<A>, BuildHasherDefault<TypeIdHasher>>,
    type_: PhantomData<V>,
}

/// A view into a single empty location in an `Map`.
pub struct VacantEntry<'a, A: ?Sized + UncheckedAnyExt, V: 'a> {
    #[cfg(all(feature = "std", not(feature = "hashbrown")))]
    inner: raw_hash_map::VacantEntry<'a, TypeId, Box<A>>,
    #[cfg(feature = "hashbrown")]
    inner: raw_hash_map::VacantEntry<'a, TypeId, Box<A>, BuildHasherDefault<TypeIdHasher>>,
    type_: PhantomData<V>,
}

/// A view into a single location in an `Map`, which may be vacant or occupied.
pub enum Entry<'a, A: ?Sized + UncheckedAnyExt, V: 'a> {
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, A, V>),
    /// A vacant Entry
    Vacant(VacantEntry<'a, A, V>),
}

impl<'a, A: ?Sized + UncheckedAnyExt, V: IntoBox<A>> Entry<'a, A, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_default(self) -> &'a mut V where V: Default {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(Default::default()),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the
    /// map.
    #[inline]
    // std::collections::hash_map::Entry::and_modify doesn’t have #[must_use], I’ll follow suit.
    #[allow(clippy::return_self_not_must_use)]
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self {
        match self {
            Entry::Occupied(mut inner) => {
                f(inner.get_mut());
                Entry::Occupied(inner)
            },
            Entry::Vacant(inner) => Entry::Vacant(inner),
        }
    }

    // Additional stable methods (as of 1.60.0-nightly) that could be added:
    // insert_entry(self, value: V) -> OccupiedEntry<'a, K, V>                             (1.59.0)
}

impl<'a, A: ?Sized + UncheckedAnyExt, V: IntoBox<A>> OccupiedEntry<'a, A, V> {
    /// Gets a reference to the value in the entry
    #[inline]
    pub fn get(&self) -> &V {
        unsafe { self.inner.get().downcast_ref_unchecked() }
    }

    /// Gets a mutable reference to the value in the entry
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.inner.get_mut().downcast_mut_unchecked() }
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the collection itself
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.inner.into_mut().downcast_mut_unchecked() }
    }

    /// Sets the value of the entry, and returns the entry's old value
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        unsafe { *self.inner.insert(value.into_box()).downcast_unchecked() }
    }

    /// Takes the value out of the entry, and returns it
    #[inline]
    pub fn remove(self) -> V {
        unsafe { *self.inner.remove().downcast_unchecked() }
    }
}

impl<'a, A: ?Sized + UncheckedAnyExt, V: IntoBox<A>> VacantEntry<'a, A, V> {
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self.inner.insert(value.into_box()).downcast_mut_unchecked() }
    }
}

/// A hasher designed to eke a little more speed out, given `TypeId`’s known characteristics.
///
/// Specifically, this is a no-op hasher that expects to be fed a u64’s worth of
/// randomly-distributed bits. It works well for `TypeId` (eliminating start-up time, so that my
/// get_missing benchmark is ~30ns rather than ~900ns, and being a good deal faster after that, so
/// that my insert_and_get_on_260_types benchmark is ~12μs instead of ~21.5μs), but will
/// panic in debug mode and always emit zeros in release mode for any other sorts of inputs, so
/// yeah, don’t use it! 😀
#[derive(Default)]
pub struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive exactly one 64-bit value, and there’s no realistic chance of
        // that changing, but I don’t want to depend on something that isn’t expressly part of the
        // contract for safety. But I’m OK with release builds putting everything in one bucket
        // if it *did* change (and debug builds panicking).
        debug_assert_eq!(bytes.len(), 8);
        let _ = bytes.try_into()
            .map(|array| self.value = u64::from_ne_bytes(array));
    }

    #[inline]
    fn finish(&self) -> u64 { self.value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)] struct A(i32);
    #[derive(Clone, Debug, PartialEq)] struct B(i32);
    #[derive(Clone, Debug, PartialEq)] struct C(i32);
    #[derive(Clone, Debug, PartialEq)] struct D(i32);
    #[derive(Clone, Debug, PartialEq)] struct E(i32);
    #[derive(Clone, Debug, PartialEq)] struct F(i32);
    #[derive(Clone, Debug, PartialEq)] struct J(i32);

    macro_rules! test_entry {
        ($name:ident, $init:ty) => {
            #[test]
            fn $name() {
                let mut map = <$init>::new();
                assert_eq!(map.insert(A(10)), None);
                assert_eq!(map.insert(B(20)), None);
                assert_eq!(map.insert(C(30)), None);
                assert_eq!(map.insert(D(40)), None);
                assert_eq!(map.insert(E(50)), None);
                assert_eq!(map.insert(F(60)), None);

                // Existing key (insert)
                match map.entry::<A>() {
                    Entry::Vacant(_) => unreachable!(),
                    Entry::Occupied(mut view) => {
                        assert_eq!(view.get(), &A(10));
                        assert_eq!(view.insert(A(100)), A(10));
                    }
                }
                assert_eq!(map.get::<A>().unwrap(), &A(100));
                assert_eq!(map.len(), 6);


                // Existing key (update)
                match map.entry::<B>() {
                    Entry::Vacant(_) => unreachable!(),
                    Entry::Occupied(mut view) => {
                        let v = view.get_mut();
                        let new_v = B(v.0 * 10);
                        *v = new_v;
                    }
                }
                assert_eq!(map.get::<B>().unwrap(), &B(200));
                assert_eq!(map.len(), 6);


                // Existing key (remove)
                match map.entry::<C>() {
                    Entry::Vacant(_) => unreachable!(),
                    Entry::Occupied(view) => {
                        assert_eq!(view.remove(), C(30));
                    }
                }
                assert_eq!(map.get::<C>(), None);
                assert_eq!(map.len(), 5);


                // Inexistent key (insert)
                match map.entry::<J>() {
                    Entry::Occupied(_) => unreachable!(),
                    Entry::Vacant(view) => {
                        assert_eq!(*view.insert(J(1000)), J(1000));
                    }
                }
                assert_eq!(map.get::<J>().unwrap(), &J(1000));
                assert_eq!(map.len(), 6);

                // Entry.or_insert on existing key
                map.entry::<B>().or_insert(B(71)).0 += 1;
                assert_eq!(map.get::<B>().unwrap(), &B(201));
                assert_eq!(map.len(), 6);

                // Entry.or_insert on nonexisting key
                map.entry::<C>().or_insert(C(300)).0 += 1;
                assert_eq!(map.get::<C>().unwrap(), &C(301));
                assert_eq!(map.len(), 7);
            }
        }
    }

    test_entry!(test_entry_any, AnyMap);
    test_entry!(test_entry_cloneany, Map<dyn CloneAny>);

    #[test]
    fn test_default() {
        let map: AnyMap = Default::default();
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_clone() {
        let mut map: Map<dyn CloneAny> = Map::new();
        let _ = map.insert(A(1));
        let _ = map.insert(B(2));
        let _ = map.insert(D(3));
        let _ = map.insert(E(4));
        let _ = map.insert(F(5));
        let _ = map.insert(J(6));
        let map2 = map.clone();
        assert_eq!(map2.len(), 6);
        assert_eq!(map2.get::<A>(), Some(&A(1)));
        assert_eq!(map2.get::<B>(), Some(&B(2)));
        assert_eq!(map2.get::<C>(), None);
        assert_eq!(map2.get::<D>(), Some(&D(3)));
        assert_eq!(map2.get::<E>(), Some(&E(4)));
        assert_eq!(map2.get::<F>(), Some(&F(5)));
        assert_eq!(map2.get::<J>(), Some(&J(6)));
    }

    #[test]
    fn test_varieties() {
        fn assert_send<T: Send>() { }
        fn assert_sync<T: Sync>() { }
        fn assert_clone<T: Clone>() { }
        fn assert_debug<T: ::core::fmt::Debug>() { }
        assert_send::<Map<dyn Any + Send>>();
        assert_send::<Map<dyn Any + Send + Sync>>();
        assert_sync::<Map<dyn Any + Send + Sync>>();
        assert_debug::<Map<dyn Any>>();
        assert_debug::<Map<dyn Any + Send>>();
        assert_debug::<Map<dyn Any + Send + Sync>>();
        assert_send::<Map<dyn CloneAny + Send>>();
        assert_send::<Map<dyn CloneAny + Send + Sync>>();
        assert_sync::<Map<dyn CloneAny + Send + Sync>>();
        assert_clone::<Map<dyn CloneAny + Send>>();
        assert_clone::<Map<dyn CloneAny + Send + Sync>>();
        assert_clone::<Map<dyn CloneAny + Send + Sync>>();
        assert_debug::<Map<dyn CloneAny>>();
        assert_debug::<Map<dyn CloneAny + Send>>();
        assert_debug::<Map<dyn CloneAny + Send + Sync>>();
    }

    #[test]
    fn type_id_hasher() {
        #[cfg(not(feature = "std"))]
        use alloc::vec::Vec;
        use core::hash::Hash;
        fn verify_hashing_with(type_id: TypeId) {
            let mut hasher = TypeIdHasher::default();
            type_id.hash(&mut hasher);
            // SAFETY: u64 is valid for all bit patterns.
            assert_eq!(hasher.finish(), unsafe { core::mem::transmute::<TypeId, u64>(type_id) });
        }
        // Pick a variety of types, just to demonstrate it’s all sane. Normal, zero-sized, unsized, &c.
        verify_hashing_with(TypeId::of::<usize>());
        verify_hashing_with(TypeId::of::<()>());
        verify_hashing_with(TypeId::of::<str>());
        verify_hashing_with(TypeId::of::<&str>());
        verify_hashing_with(TypeId::of::<Vec<u8>>());
    }
}
