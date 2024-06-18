//! Sorted vector implementation using the _allocated_ pattern.
//!
//! This module provides:
//! - [`AllocatedSortedVec<T>`]: Sorted vector without owned allocator
//! - [`SortedVec<T, A>`]: Ergonomic wrapper with owned allocator
//!
//! A sorted vector maintains elements in sorted order, enabling efficient
//! binary search operations. Elements must implement [`Ord`].
//!
//! # Relationship to `vec` module
//!
//! `AllocatedSortedVec` is built on top of `AllocatedVec`, demonstrating
//! how the allocated pattern enables composition. The inner `AllocatedVec`
//! is stored in a `ManuallyDrop` wrapper, allowing both to share a single
//! allocator instance in the wrapper type.
//!
//! # Example
//!
//! ```
//! use allocated::SortedVec;
//!
//! let mut v = SortedVec::new();
//! v.insert(3);
//! v.insert(1);
//! v.insert(2);
//! assert_eq!(&v[..], &[1, 2, 3]);
//! ```

mod _allocated;
mod _impl;

pub use _allocated::AllocatedSortedVec;
pub use _impl::SortedVec;
