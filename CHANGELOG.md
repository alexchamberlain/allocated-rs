# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added
- Initial release of the `allocated` crate
- Core traits: `DropIn`, `RecursiveDropIn`, `FromIteratorIn`, `CollectIn`, `IntoIteratorIn`
- Safety utilities: `DropGuard` and `RawDropGuard` for preventing memory leaks
- `AllocatorExt` trait with convenience methods for single and array allocations
- Data structures using the allocated pattern:
  - `Vec` and `AllocatedVec`: Dynamic array implementation
  - `SortedVec` and `AllocatedSortedVec`: Sorted vector with binary search
  - `VectorMap` and `AllocatedVectorMap`: Map using parallel sorted arrays
- Utility allocators:
  - `CountingAllocator`: Track allocation counts and bytes
  - `TrackingAllocator`: Track active allocations with backtraces
- Comprehensive module-level documentation
- Examples demonstrating the allocated pattern

[Unreleased]: https://github.com/alexchamberlain/allocated-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/alexchamberlain/allocated-rs/releases/tag/v0.1.0
