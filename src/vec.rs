//! Dynamic array implementation using the _allocated_ pattern.
//!
//! This module provides two types:
//! - [`AllocatedVec<T>`]: The low-level type that doesn't own an allocator
//! - [`Vec<T, A>`]: The ergonomic wrapper that owns both data and allocator
//!
//! # When to use which type
//!
//! **Use [`Vec<T, A>`]** (the wrapper) for:
//! - Application code where you need a standalone vector
//! - When you want automatic cleanup (Drop)
//! - API boundaries and public interfaces
//!
//! **Use [`AllocatedVec<T>`]** (the allocated type) for:
//! - Building composite data structures (like `VectorMap`)
//! - Avoiding allocator duplication in nested structures
//! - When you need manual control over deallocation order
//!
//! # Example
//!
//! ```
//! use allocated::Vec;
//! use allocator_api2::alloc::Global;
//!
//! // Using the wrapper type (most common)
//! let mut v = Vec::new();
//! v.push(1);
//! v.push(2);
//! v.push(3);
//! assert_eq!(v.len(), 3);
//!
//! // Using a custom allocator
//! let mut v = Vec::new_in(Global).unwrap();
//! v.push("hello");
//! ```
//!
//! # Safety
//!
//! The [`AllocatedVec`] type has methods ending in `_in` that are marked `unsafe`.
//! This is because they require you to pass the **same allocator instance** that
//! was used to create the vector. The wrapper type [`Vec`] enforces this invariant
//! through ownership, making all its methods safe.

mod _allocated;
mod _impl;
mod _raw_vec;

pub use _allocated::{AllocatedVec, Drain};
pub use _impl::{SliceExt, Vec};
