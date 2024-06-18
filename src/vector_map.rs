//! Map implementation using sorted, dynamic arrays.
//!
//! This module provides:
//! - [`AllocatedVectorMap<K, V>`]: Map without owned allocator
//! - [`VectorMap<K, V, A>`]: Ergonomic wrapper with owned allocator
//!
//! A `VectorMap` stores key-value pairs in two parallel sorted arrays,
//! providing O(log n) lookup via binary search and O(n) insertion/removal.
//! This is space-efficient for small to medium-sized maps.
//!
//! # Composition example
//!
//! This type demonstrates the power of the allocated pattern:
//! ```rust,ignore
//! pub struct AllocatedVectorMap<K, V> {
//!     keys: ManuallyDrop<AllocatedVec<K>>,
//!     values: ManuallyDrop<AllocatedVec<V>>,
//! }
//! ```
//!
//! The wrapper `VectorMap` stores a single allocator that is used for both
//! internal vectors, avoiding allocator duplication while maintaining safety.
//!
//! # Example
//!
//! ```
//! use allocated::VectorMap;
//!
//! let mut map = VectorMap::new();
//! map.insert("key", 42);
//! assert_eq!(map.get(&"key"), Some(&42));
//! assert_eq!(map.len(), 1);
//!
//! // Replacing a value returns the old one
//! assert_eq!(map.insert("key", 100), Some(42));
//! assert_eq!(map.get(&"key"), Some(&100));
//! ```
//!
//! # Performance characteristics
//!
//! - Insertion: O(n) - requires shifting elements
//! - Lookup: O(log n) - binary search
//! - Removal: O(n) - requires shifting elements
//! - Memory: Very compact, no per-entry overhead

mod _allocated;
mod _impl;

pub use _allocated::{
    AllocatedVectorMap, Entry, Iter, IterMut, Keys, OccupiedEntry, VacantEntry, Values, ValuesMut,
};
pub use _impl::VectorMap;
