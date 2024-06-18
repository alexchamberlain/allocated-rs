#![doc = include_str!("../DOCS.md")]

mod _allocator_ext;
mod _counting_allocator;
mod _drop_guard;
mod _error;
mod _tracking_allocator;
mod _traits;
pub mod sorted_vec;
pub mod vec;
pub mod vector_map;

pub use _allocator_ext::AllocatorExt;
pub use _counting_allocator::CountingAllocator;
pub use _drop_guard::{DropGuard, DropGuardResult, RawDropGuard, RawDropGuardResult};
pub use _error::{AllocErrorWithLayout, AllocResult, AllocResultExt};
pub use _tracking_allocator::TrackingAllocator;
pub use _traits::{CollectIn, DropIn, FromIteratorIn, IntoIteratorIn, RecursiveDropIn};
pub use sorted_vec::{AllocatedSortedVec, SortedVec};
pub use vec::{AllocatedVec, SliceExt, Vec};
pub use vector_map::{AllocatedVectorMap, VectorMap};
