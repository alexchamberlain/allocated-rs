//! Implementation of a map using sorted, dynamic arrays.

use core::cmp::Ord;
use core::fmt::Debug;
use core::mem::ManuallyDrop;

use allocator_api2::alloc::{Allocator, Global};

use crate::_error::{AllocResult, AllocResultExt};
use crate::_traits::DropIn;

use super::_allocated::{AllocatedVectorMap, Entry, Iter, IterMut, Keys, Values, ValuesMut};

/// A map using sorted dynamic arrays
#[derive(Debug)]
pub struct VectorMap<K, V, A: Allocator = Global>
where
    K: Ord,
{
    alloc: A,
    raw: ManuallyDrop<AllocatedVectorMap<K, V>>,
}

impl<K: Ord, V: PartialEq, A: Allocator> PartialEq<Self> for VectorMap<K, V, A> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<K: Ord, V: PartialEq, A: Allocator> Eq for VectorMap<K, V, A> {}

impl<K: Ord, V, A: Allocator> Drop for VectorMap<K, V, A> {
    fn drop(&mut self) {
        // Safety: `raw` was allocated using `alloc`
        unsafe { self.raw.drop_in(&self.alloc) };
    }
}

impl<K: Ord, V> VectorMap<K, V> {
    #[inline]
    pub fn new() -> Self {
        let raw = AllocatedVectorMap::new_in(&Global)
            .handle_alloc_error()
            .into_inner();

        Self { alloc: Global, raw }
    }
}

impl<K: Ord, V> Default for VectorMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V, A: Allocator> VectorMap<K, V, A> {
    pub fn with_capacity_in(alloc: A, capacity: usize) -> AllocResult<Self> {
        let raw = AllocatedVectorMap::with_capacity_in(&alloc, capacity)?.into_inner();
        Ok(Self { alloc, raw })
    }

    pub fn bind(alloc: A, raw: AllocatedVectorMap<K, V>) -> Self {
        let raw = ManuallyDrop::new(raw);
        Self { alloc, raw }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    pub fn clear(&mut self) {
        self.raw.clear()
    }

    pub fn keys(&self) -> Keys<'_, K> {
        self.raw.keys()
    }
    pub fn values(&self) -> Values<'_, V> {
        self.raw.values()
    }
    pub fn values_mut(&mut self) -> ValuesMut<'_, V> {
        self.raw.values_mut()
    }
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.raw.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.raw.iter_mut()
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // SAFETY: self.alloc is always used for self.raw's allocations
        unsafe { self.raw.insert_in(&self.alloc, k, v).unwrap() }
    }

    pub fn remove(&mut self, k: K) -> Option<V> {
        self.raw.remove(k)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.raw.get(k)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.raw.get_mut(k)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.raw.get(k).is_some()
    }

    pub fn entry(&mut self, k: K) -> Entry<'_, K, V, A> {
        // SAFETY: self.alloc is always used for self.raw's allocations
        unsafe { self.raw.entry_in(&self.alloc, k) }
    }

    pub fn reserve(&mut self, additional: usize) {
        // SAFETY: self.alloc is always used for self.raw's allocations
        unsafe { self.raw.reserve_in(&self.alloc, additional) }
    }

    pub fn shrink_to_fit(&mut self) {
        // SAFETY: self.alloc is always used for self.raw's allocations
        unsafe { self.raw.shrink_to_fit_in(&self.alloc).handle_alloc_error() }
    }

    pub fn first(&self) -> Option<(&K, &V)> {
        self.raw.first()
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        self.raw.pop()
    }
}

#[cfg(test)]
mod test {
    use std::vec::Vec;

    use crate::CountingAllocator;

    use super::AllocResult;
    use super::VectorMap;

    #[test]
    fn test_new() -> AllocResult<()> {
        let m: VectorMap<u32, u32> = VectorMap::new();
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());

        Ok(())
    }

    #[test]
    fn test_insert() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 42), None);
        assert_eq!(m.insert(10, 54), Some(42));

        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(m.len(), 4);

        let c = m.capacity();
        m.clear();
        assert_eq!(m.len(), 0);
        assert_eq!(m.capacity(), c);
        assert!(m.is_empty());

        Ok(())
    }

    #[test]
    fn test_iter() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(
            m.iter().map(|(&k, &v)| (k, v)).collect::<Vec<(u32, u32)>>(),
            vec![(1, 42), (2, 100), (10, 54), (54, 2)]
        );

        Ok(())
    }

    #[test]
    fn test_keys() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(m.keys().copied().collect::<Vec<u32>>(), vec![1, 2, 10, 54]);

        Ok(())
    }

    #[test]
    fn test_values() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(
            m.values().copied().collect::<Vec<u32>>(),
            vec![42, 100, 54, 2]
        );

        Ok(())
    }

    #[test]
    fn test_remove() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.remove(1), Some(42));
        assert_eq!(m.insert(54, 2), None);
        assert_eq!(m.remove(99), None);

        assert_eq!(
            m.iter().map(|(&k, &v)| (k, v)).collect::<Vec<(u32, u32)>>(),
            vec![(2, 100), (10, 54), (54, 2)]
        );

        Ok(())
    }

    #[test]
    fn test_iter_mut() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        for (_k, v) in m.iter_mut() {
            *v *= 2;
        }

        assert_eq!(
            m.iter().map(|(&k, &v)| (k, v)).collect::<Vec<(u32, u32)>>(),
            vec![(1, 84), (2, 200), (10, 108), (54, 4)]
        );

        Ok(())
    }

    #[test]
    fn test_values_mut() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(
            m.values_mut().map(|&mut k| k).collect::<Vec<u32>>(),
            vec![42, 100, 54, 2]
        );

        for v in m.values_mut() {
            *v *= 2;
        }

        assert_eq!(
            m.iter().map(|(&k, &v)| (k, v)).collect::<Vec<(u32, u32)>>(),
            vec![(1, 84), (2, 200), (10, 108), (54, 4)]
        );

        Ok(())
    }

    #[test]
    fn test_allocations() -> AllocResult<()> {
        let alloc = CountingAllocator::default();
        let mut m: VectorMap<u32, u32, _> = VectorMap::with_capacity_in(alloc, 4)?;

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);
        assert_eq!(m.insert(54, 2), None);

        assert_eq!(m.alloc.n_allocations(), 2);

        Ok(())
    }

    #[test]
    fn test_get() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);

        assert_eq!(m.get(&1), Some(&42));
        assert_eq!(m.get(&10), Some(&54));
        assert_eq!(m.get(&99), None);

        Ok(())
    }

    #[test]
    fn test_get_mut() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);

        if let Some(v) = m.get_mut(&10) {
            *v = 100;
        }

        assert_eq!(m.get(&10), Some(&100));
        assert_eq!(m.get_mut(&99), None);

        Ok(())
    }

    #[test]
    fn test_contains_key() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);

        assert!(m.contains_key(&10));
        assert!(m.contains_key(&1));
        assert!(!m.contains_key(&99));

        Ok(())
    }

    #[test]
    fn test_entry() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        m.entry(10).or_insert(54);
        m.entry(1).or_insert(42);

        assert_eq!(m.get(&10), Some(&54));
        assert_eq!(m.get(&1), Some(&42));

        *m.entry(10).or_insert(0) = 100;
        assert_eq!(m.get(&10), Some(&100));

        Ok(())
    }

    #[test]
    fn test_first_and_pop() -> AllocResult<()> {
        let mut m: VectorMap<u32, u32> = VectorMap::new();

        assert_eq!(m.first(), None);
        assert_eq!(m.pop(), None);

        assert_eq!(m.insert(10, 54), None);
        assert_eq!(m.insert(1, 42), None);
        assert_eq!(m.insert(2, 100), None);

        // First returns the smallest key
        assert_eq!(m.first(), Some((&1, &42)));

        // Pop removes the largest key
        assert_eq!(m.pop(), Some((10, 54)));
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop(), Some((2, 100)));
        assert_eq!(m.pop(), Some((1, 42)));
        assert_eq!(m.pop(), None);

        Ok(())
    }
}
