use core::cmp::Ord;
use core::fmt::Debug;
use core::mem;
use core::mem::ManuallyDrop;
use core::ops::Index;

use allocator_api2::alloc::Allocator;

use crate::_drop_guard::{DropGuard, DropGuardResult};
use crate::_error::AllocResult;
use crate::_traits::{DropIn, RecursiveDropIn};
use crate::vec::AllocatedVec;

/// An _allocated_ map using sorted dynamic arrays
#[derive(Debug, PartialEq, Eq)]
pub struct AllocatedVectorMap<K, V>
where
    K: Ord,
{
    keys: ManuallyDrop<AllocatedVec<K>>,
    values: ManuallyDrop<AllocatedVec<V>>,
}

impl<K: Ord, V> AllocatedVectorMap<K, V> {
    /// Create a new, empty `AllocatedVectorMap` using the given
    /// allocator.
    pub fn new_in<A: Allocator>(alloc: &A) -> DropGuardResult<Self, &A> {
        let keys = AllocatedVec::try_with_capacity_in(alloc, 0)?;
        let values = AllocatedVec::try_with_capacity_in(alloc, 0)?;

        // Safety: alloc was used for both vecs above
        Ok(unsafe {
            DropGuard::new(
                Self {
                    keys: keys.into_inner(),
                    values: values.into_inner(),
                },
                alloc,
            )
        })
    }

    /// Create a new, empty `AllocatedVectorMap` using the given
    /// allocator, with enough space for `capacity` entries.
    pub fn with_capacity_in<A: Allocator>(alloc: &A, capacity: usize) -> DropGuardResult<Self, &A> {
        let keys = AllocatedVec::try_with_capacity_in(alloc, capacity)?;
        let values = AllocatedVec::try_with_capacity_in(alloc, capacity)?;

        // Safety: alloc was used for both vecs above
        Ok(unsafe {
            DropGuard::new(
                Self {
                    keys: keys.into_inner(),
                    values: values.into_inner(),
                },
                alloc,
            )
        })
    }

    /// Create a new `AllocatedVectorMap` using the given
    /// allocator, with a single entry.
    pub fn one_in<A: Allocator>(alloc: &A, k: K, v: V) -> DropGuardResult<Self, &A> {
        let mut s = Self::with_capacity_in(alloc, 1)?;

        // Safety: alloc was used to create s above
        unsafe { s.keys.insert_in(alloc, 0, k)? };
        // Safety: alloc was used to create s above
        unsafe { s.values.insert_in(alloc, 0, v)? };

        Ok(s)
    }

    /// Create a new `AllocatedVectorMap` by zipping together keys and values.
    ///
    /// # Safety
    ///
    /// `keys` and `values` must be allocated using the same allocator.
    #[inline]
    pub unsafe fn zip(keys: AllocatedVec<K>, values: AllocatedVec<V>) -> AllocatedVectorMap<K, V> {
        assert_eq!(keys.len(), values.len());

        AllocatedVectorMap {
            keys: ManuallyDrop::new(keys),
            values: ManuallyDrop::new(values),
        }
    }

    /// Extend the map with new keys and values.
    ///
    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    #[inline]
    pub unsafe fn extend_with_parts<
        A: Allocator,
        KS: IntoIterator<Item = K>,
        VS: IntoIterator<Item = V>,
    >(
        &mut self,
        alloc: &A,
        keys: KS,
        values: VS,
    ) -> AllocResult<()>
    where
        Self: Sized,
    {
        self.keys.extend_in(alloc, keys)?;
        self.values.extend_in(alloc, values)?;

        Ok(())
    }

    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    #[inline]
    pub unsafe fn reserve_in<A: Allocator>(&mut self, alloc: &A, additional: usize) {
        self.keys.try_reserve_in(alloc, additional).unwrap();
        self.values.try_reserve_in(alloc, additional).unwrap();
    }

    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    #[inline]
    pub unsafe fn shrink_to_fit_in<A: Allocator>(&mut self, alloc: &A) -> AllocResult<()> {
        self.keys.shrink_to_fit_in(alloc)?;
        self.values.shrink_to_fit_in(alloc)?;

        Ok(())
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.keys.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    pub fn keys(&self) -> Keys<'_, K> {
        Keys {
            iter: self.keys.iter(),
        }
    }

    pub fn values(&self) -> Values<'_, V> {
        Values {
            iter: self.values.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, V> {
        ValuesMut {
            iter: self.values.iter_mut(),
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.keys.iter().zip(self.values.iter()),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.keys.iter().zip(self.values.iter_mut()),
        }
    }

    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    #[inline]
    pub unsafe fn insert_in<A: Allocator>(
        &mut self,
        alloc: &A,
        k: K,
        v: V,
    ) -> AllocResult<Option<V>> {
        match self.keys.binary_search(&k) {
            Ok(pos) => Ok(Some(mem::replace(&mut self.values[pos], v))),
            Err(pos) => {
                self.keys.insert_in(alloc, pos, k)?;
                self.values.insert_in(alloc, pos, v)?;
                Ok(None)
            }
        }
    }

    pub fn remove(&mut self, k: K) -> Option<V> {
        match self.keys.binary_search(&k) {
            Ok(pos) => {
                self.keys.remove(pos);
                Some(self.values.remove(pos))
            }
            Err(_pos) => None,
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        match self.keys.binary_search(k) {
            Ok(pos) => Some(&self.values[pos]),
            Err(_pos) => None,
        }
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        match self.keys.binary_search(k) {
            Ok(pos) => Some(&mut self.values[pos]),
            Err(_pos) => None,
        }
    }

    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    pub unsafe fn entry_in<'a, A: Allocator>(
        &'a mut self,
        alloc: &'a A,
        k: K,
    ) -> Entry<'a, K, V, A> {
        match self.keys.binary_search(&k) {
            Ok(pos) => Entry::Occupied(OccupiedEntry::new(self, pos)),
            Err(pos) => Entry::Vacant(VacantEntry::new(self, alloc, pos, k)),
        }
    }

    #[inline]
    pub fn first(&self) -> Option<(&K, &V)> {
        self.keys.first().map(|k| (k, self.values.first().unwrap()))
    }

    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.keys.pop().map(|k| (k, self.values.pop().unwrap()))
    }
}

impl<K: Ord, V> Drop for AllocatedVectorMap<K, V> {
    fn drop(&mut self) {
        panic!("AllocatedVectorMap MUST not be dropped. Use drop_in instead.")
    }
}

impl<K: Ord, V> DropIn for AllocatedVectorMap<K, V> {
    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object.
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A) {
        for k in self.keys.iter_mut() {
            core::ptr::drop_in_place(k);
        }

        for v in self.values.iter_mut() {
            core::ptr::drop_in_place(v);
        }

        self.keys.drop_in(alloc);
        self.values.drop_in(alloc);
    }
}

impl<K: Ord, V: DropIn> RecursiveDropIn for AllocatedVectorMap<K, ManuallyDrop<V>> {
    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate this object, as well as
    /// the values.
    unsafe fn recursive_drop_in<A: Allocator>(&mut self, alloc: &A) {
        while let Some((_, mut v)) = self.pop() {
            v.drop_in(alloc);
        }
        self.keys.drop_in(alloc);
        self.values.drop_in(alloc);
    }
}

impl<K: Ord, V> Index<&K> for AllocatedVectorMap<K, V> {
    type Output = V;

    fn index(&self, k: &K) -> &Self::Output {
        match self.keys.binary_search(k) {
            Ok(pos) => &self.values[pos],
            Err(_pos) => {
                // println!("k={:?} keys={:?}", k, self.keys);
                panic!("Key does not exist");
            }
        }
    }
}
#[derive(Debug)]
pub enum Entry<'a, K: Ord, V, A: Allocator> {
    Vacant(VacantEntry<'a, K, V, A>),
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K: Ord, V, A: Allocator> Entry<'a, K, V, A> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a, K: Ord, V, A: Allocator> {
    this: &'a mut AllocatedVectorMap<K, V>,
    alloc: &'a A,
    pos: usize,
    k: K,
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, K: Ord, V> {
    this: &'a mut AllocatedVectorMap<K, V>,
    pos: usize,
}

impl<'a, K: 'a + Ord, V: 'a, A: Allocator> VacantEntry<'a, K, V, A> {
    /// # Safety
    ///
    /// `alloc` must be the allocator used to allocate `this`.
    #[inline]
    pub unsafe fn new(
        this: &'a mut AllocatedVectorMap<K, V>,
        alloc: &'a A,
        pos: usize,
        k: K,
    ) -> VacantEntry<'a, K, V, A> {
        VacantEntry {
            this,
            alloc,
            pos,
            k,
        }
    }

    #[inline]
    pub fn insert(self, v: V) -> &'a mut V {
        // Safety: asserted by unsafe new function
        unsafe {
            self.this
                .keys
                .insert_in(self.alloc, self.pos, self.k)
                .unwrap()
        };
        // Safety: asserted by unsafe new function
        unsafe { self.this.values.insert_in(self.alloc, self.pos, v).unwrap() };

        &mut self.this.values[self.pos]
    }
}

impl<'a, K: 'a + Ord, V: 'a> OccupiedEntry<'a, K, V> {
    #[inline]
    pub fn new(this: &'a mut AllocatedVectorMap<K, V>, pos: usize) -> OccupiedEntry<'a, K, V> {
        OccupiedEntry { this, pos }
    }

    #[inline]
    pub fn get(&self) -> &V {
        &self.this.values[self.pos]
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.this.values[self.pos]
    }

    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        &mut self.this.values[self.pos]
    }

    #[inline]
    pub fn remove_entry(&mut self) -> (K, V) {
        (
            self.this.keys.remove(self.pos),
            self.this.values.remove(self.pos),
        )
    }
}

pub struct Keys<'a, K> {
    iter: core::slice::Iter<'a, K>,
}

impl<'a, K: 'a> Iterator for Keys<'a, K> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: 'a> DoubleEndedIterator for Keys<'a, K> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

pub struct Values<'a, V> {
    iter: core::slice::Iter<'a, V>,
}

impl<'a, K: 'a> Iterator for Values<'a, K> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: 'a> DoubleEndedIterator for Values<'a, K> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

pub struct ValuesMut<'a, V> {
    iter: core::slice::IterMut<'a, V>,
}

impl<'a, K: 'a> Iterator for ValuesMut<'a, K> {
    type Item = &'a mut K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: 'a> DoubleEndedIterator for ValuesMut<'a, K> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

pub struct Iter<'a, K, V> {
    iter: core::iter::Zip<core::slice::Iter<'a, K>, core::slice::Iter<'a, V>>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

pub struct IterMut<'a, K, V> {
    iter: core::iter::Zip<core::slice::Iter<'a, K>, core::slice::IterMut<'a, V>>,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
