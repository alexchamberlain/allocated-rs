# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`allocated` is a Rust crate that provides abstractions for working with explicitly allocated data structures. It enables ergonomic and memory-safe data structures by separating allocation concerns from data structure logic.

## Core Architecture

### The Allocated Pattern

The crate uses a two-struct design pattern:

1. **Allocated Struct** (e.g., `AllocatedVec`, `AllocatedSortedVec`, `AllocatedVectorMap`)
   - Does NOT own an `Allocator` instance
   - Implements `DropIn` trait for custom drop semantics
   - Allows reuse within other data structures
   - Located in `_allocated.rs` files within each module

2. **Wrapper Struct** (e.g., `Vec`, `SortedVec`, `VectorMap`)
   - Partners the allocated struct with an `Allocator`
   - Provides ergonomic API for application use
   - Located in `_impl.rs` files within each module

### Key Traits (src/\_traits.rs)

- **`DropIn`**: Drop elements with allocator context (equivalent to `std::ops::Drop`)
- **`RecursiveDropIn`**: Recursively drop elements and container
- **`FromIteratorIn`**: Create containers from iterators using a specific allocator
- **`CollectIn`**: Transform iterators into allocated collections
- **`IntoIteratorIn`**: Convert into iterator using a given allocator

### Safety Utilities

- **`DropGuard`** (src/\_drop\_guard.rs): Prevents memory leaks during fallible operations by automatically calling `drop_in` on scope exit
- **`RawDropGuard`**: Similar to `DropGuard` but for raw pointers
- **`AllocatorExt`** (src/\_allocator\_ext.rs): Extension trait providing convenience methods for allocating arrays and single instances

### Data Structures

- **`AllocatedVec`** (src/vec/): Dynamic array based on The Rustonomicon
- **`AllocatedSortedVec`** (src/sorted_vec/): Sorted variant of Vec
- **`AllocatedVectorMap`** (src/vector_map/): Map implementation using sorted arrays

## Development Commands

### Build and Check
```bash
cargo build          # Build the project
cargo check          # Fast compilation check
cargo clippy         # Run linter (with private items checked per clippy.toml)
```

### Testing
```bash
cargo test                    # Run all tests (no_std mode, 16 tests)
cargo test --features std     # Run with std feature (18 tests, includes TrackingAllocator)
cargo test <name>             # Run specific test
```

### Documentation
```bash
cargo doc --open     # Build and open documentation
```

## Naming Conventions

- Methods taking an `Allocator` instance end with `_in` suffix (e.g., `new_in`, `push_in`, `insert_in`)
- The allocator parameter always comes first in `_in` methods
- Wrapper structs provide safe equivalents without the `_in` suffix

## Safety Requirements

### Creating Allocated Structures
- Methods like `new_in` should:
  - Take an `Allocator` instance
  - Return a `DropGuardResult` (an `AllocResult<DropGuard<T, A>>`)

### Mutating Operations
- Methods that may allocate (e.g., `insert_in`, `push_in`, `extend_in`) are marked `unsafe` because:
  - They must receive the SAME `Allocator` instance used to create the structure
  - They return an `AllocResult`

### Critical Invariant
When calling any `_in` method, the allocator MUST be the same instance used to allocate the structure. Violating this is undefined behavior.

## Clippy Configuration

The project enforces:
- `undocumented_unsafe_blocks = "warn"`
- `multiple_unsafe_ops_per_block = "warn"`
- `check-private-items = true` (in clippy.toml)

All unsafe blocks must be documented with safety comments explaining why they are safe.

## no_std Compatibility

This crate is **`no_std` by default** to support embedded systems, kernels, and WebAssembly.

### Feature Flags

- **Default (no features)**: Core functionality using `core` and `alloc`
  - `Vec`, `SortedVec`, `VectorMap`
  - `DropGuard`, `AllocatorExt`
  - `CountingAllocator`

- **`std` feature**: Enables std-dependent functionality
  - `TrackingAllocator` (uses `Backtrace` and `HashMap`)
  - `HashSet` comparison impl for `AllocatedSortedVec`

### Import Conventions for no_std

- Use `core::` for primitives: `mem`, `ptr`, `ops`, `cmp`, `fmt`
- Use `alloc::` for heap types: `rc::Rc`, `vec::Vec`, `string::String`
- Use `std::` only behind `#[cfg(feature = "std")]`
- Test modules need `use std::vec::Vec;` for the `vec![]` macro

### Testing with std

The lib.rs has:
```rust
#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;
```

This enables the `vec![]` macro in tests. Test modules should add `use std::vec::Vec;` for the type.

## Publishing Workflow

### Pre-publish Checklist
```bash
# Test both configurations
cargo test --quiet
cargo test --features std --quiet

# Run clippy on both
cargo clippy -- -D warnings
cargo clippy --features std -- -D warnings

# Build docs
cargo doc --no-deps

# Dry run publish
cargo publish --dry-run
```

### Version Bump
1. Update version in `Cargo.toml`
2. Add entry to `CHANGELOG.md`
3. Create git tag: `git tag v0.x.y`
4. Push with tags: `git push --tags`

## Dependencies

- `allocator-api2`: Provides stable access to Rust's allocator API
