Abstractions for working with allocated data structures

`allocated` provides various utilities for working with
explicitly allocated data structures, which makes them
more ergonomic to write, easier to make them correctly
manage memory and more consistent between different implementations.

# Design

The basic idea of this crate is to separate data structures into
2 structs: an _allocated_ struct, which doesn't own an [Allocator]
instance, and a wrapper struct, which partners the allocated
struct with an [Allocator]. The first struct allows us to reuse
the data structure within a different data structure, whilst
the second makes it ergonomic to use in an application setting.

For example, [AllocatedVec] provides a basic [std::vec::Vec] equivalent
in this model - it is wrapped by a second struct [Vec] to make it
ergonomic to use in an application, whilst also being reused by
[AllocatedSortedVec] in a sorted vec implementation. [AllocatedVec],
[AllocatedSortedVec] and [AllocatedVectorMap], along with their matching
structs, serve as 3 simple examples of this model, whilst also being
useful in their own right.

Further traits and utility structs, such as [DropIn] and [DropGuard]
are provided to make it easier to write safe, efficient data
structures.

## Principles

If you want to develop a new data structure, here are some principles
to follow:

- Any method that creates the allocated data structure, such as `new_in`,
  should take an [Allocator] instance return a [DropGuardResult].
- Any method that may lead to further allocations, such as `insert_in`,
  should be marked `unsafe` on account that it has to be passed
  the same [Allocator] instance and return an [AllocResult].

As a matter of style, any method that takes an [Allocator] instance
ends in `_in` and takes the allocator first. Furthermore, safe
equivalent functions of the same name, but without the `_in` suffix,
are provided on the wrapper struct.

# Motivation

I wanted to experiment with writing a data structure that should
have been a more compressed equivalent of another data structure. To
do that, I wanted to learn about the new [Allocator] API and 
trace the allocations across the 2 implementations. I found that
this was harder than I expected, and slowly developed the ideas in this library.

[Allocator]: allocator_api2::alloc::Allocator
