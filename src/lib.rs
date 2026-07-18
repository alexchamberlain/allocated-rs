#![doc = include_str!("../DOCS.md")]
#![no_std]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

extern crate alloc;

mod _allocator_ext;
mod _counting_allocator;
mod _drop_guard;
mod _error;
#[cfg(feature = "std")]
mod _tracking_allocator;
mod _traits;
pub mod sorted_vec;
pub mod vec;
pub mod vector_map;

pub use _allocator_ext::AllocatorExt;
pub use _counting_allocator::CountingAllocator;
pub use _drop_guard::{DropGuard, DropGuardResult, RawDropGuard, RawDropGuardResult};
pub use _error::{AllocErrorWithLayout, AllocResult, AllocResultExt};
#[cfg(feature = "std")]
pub use _tracking_allocator::{SharedTrackingAllocator, TrackingAllocator};
pub use _traits::{CollectIn, DropIn, FromIteratorIn, IntoIteratorIn, RecursiveDropIn};
pub use sorted_vec::{AllocatedSortedVec, SortedVec};
pub use vec::{AllocatedVec, SliceExt, Vec};
pub use vector_map::{AllocatedVectorMap, VectorMap};
