//! This crate provides the `AnyMap` type, a safe and convenient store for one value of each type.

#![crate_id = "anymap#0.9"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![feature(default_type_params)]
#![warn(unnecessary_qualification, non_uppercase_statics,
        variant_size_difference, managed_heap_memory, unnecessary_typecast,
        missing_doc, unused_result, deprecated_owned_vector)]

#[cfg(test)]
extern crate test;

use std::any::{Any, AnyRefExt, AnyMutRefExt};
use std::intrinsics::TypeId;
use std::collections::HashMap;
use std::hash::{Hash, Hasher, Writer};

struct TypeIdHasher;

struct TypeIdState {
    value: u64,
}

impl Writer for TypeIdState {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive one and exactly one 64-bit value
        debug_assert!(bytes.len() == 8);
        unsafe {
            std::ptr::copy_nonoverlapping_memory(&mut self.value,
                                                 std::mem::transmute(&bytes[0]),
                                                 1)
        }
    }
}

impl Hasher<TypeIdState> for TypeIdHasher {
    fn hash<T: Hash<TypeIdState>>(&self, value: &T) -> u64 {
        let mut state = TypeIdState {
            value: 0,
        };
        value.hash(&mut state);
        state.value
    }
}

/// A map containing zero or one values for any given type and allowing convenient,
/// type-safe access to those values.
///
/// ```rust
/// # use anymap::AnyMap;
/// let mut data = AnyMap::new();
/// assert_eq!(data.find(), None::<&int>);
/// data.insert(42i);
/// assert_eq!(data.find(), Some(&42i));
/// data.remove::<int>();
/// assert_eq!(data.find::<int>(), None);
///
/// #[deriving(PartialEq, Show)]
/// struct Foo {
///     str: String,
/// }
///
/// assert_eq!(data.find::<Foo>(), None);
/// data.insert(Foo { str: "foo".to_string() });
/// assert_eq!(data.find(), Some(&Foo { str: "foo".to_string() }));
/// data.find_mut::<Foo>().map(|foo| foo.str.push_char('t'));
/// assert_eq!(data.find::<Foo>().unwrap().str.as_slice(), "foot");
/// ```
///
/// Values containing non-static references are not permitted.
pub struct AnyMap {
    data: HashMap<TypeId, Box<Any>:'static, TypeIdHasher>,
}

impl AnyMap {
    /// Construct a new `AnyMap`.
    pub fn new() -> AnyMap {
        AnyMap {
            data: HashMap::with_hasher(TypeIdHasher),
        }
    }
}

impl AnyMap {
    /// Retrieve the value stored in the map for the type `T`, if it exists.
    pub fn find<'a, T: 'static>(&'a self) -> Option<&'a T> {
        self.data.find(&TypeId::of::<T>()).and_then(|any| any.as_ref::<T>())
    }

    /// Retrieve a mutable reference to the value stored in the map for the type `T`, if it exists.
    pub fn find_mut<'a, T: 'static>(&'a mut self) -> Option<&'a mut T> {
        self.data.find_mut(&TypeId::of::<T>()).and_then(|any| any.as_mut::<T>())
    }

    /// Set the value contained in the map for the type `T`.
    /// This will override any previous value stored.
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), box value as Box<Any>:'static);
    }

    /// Remove the value for the type `T` if it existed.
    pub fn remove<T: 'static>(&mut self) {
        self.data.remove(&TypeId::of::<T>());
    }
}

#[bench]
fn bench_insertion(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        data.insert(42i);
    })
}

#[bench]
fn bench_find_missing(b: &mut ::test::Bencher) {
    b.iter(|| {
        let data = AnyMap::new();
        assert_eq!(data.find(), None::<&int>);
    })
}

#[bench]
fn bench_find_present(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        data.insert(42i);
        assert_eq!(data.find(), Some(&42i));
    })
}
