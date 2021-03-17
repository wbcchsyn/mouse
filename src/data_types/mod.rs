// Copyright 2021 Shin Yoshida
//
// This file is part of Mouse.
//
// Mouse is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License.
//
// Mouse is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Mouse.  If not, see <https://www.gnu.org/licenses/>.

//! `data_types` declares traits and structs for mouse data.
//! This module is independent from other modules.

mod acid;
pub mod crypto_hash;
mod resource;

use crate::{Config, ModuleEnvironment};
pub use acid::{Acid, CAcid, Id};
use clap::App;
use core::iter::IntoIterator;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::{Iter, IterMut, SliceIndex};
pub use crypto_hash::{CryptoHash, CryptoHasher};
pub use resource::{Resource, ResourceId, RESOURCE_ID_BUFFER_CAPACITY};
use std::borrow::{Borrow, BorrowMut};
use std::error::Error;

/// `Environment` implements `ModuleEnvironment` .
pub struct Environment {}

impl Default for Environment {
    fn default() -> Self {
        Self {}
    }
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app
    }

    unsafe fn check(&mut self, _: &Config) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

/// Function type to deserialize `Acid` .
pub type AcidDeserializer = fn(&[u8]) -> Result<CAcid, Box<dyn Error>>;

fn default_acid_deserializer(_: &[u8]) -> Result<CAcid, Box<dyn Error>> {
    Err(Box::from("Not specified how to deserialize 'Acid'."))
}

/// `CAlloc` implements `GlobalAlloc` and behaves like `std::alloc::System` except for that
/// `CAlloc` increases/decreases the caching byte size as allocate/deallocate heap memory.
pub use mouse_cache_alloc::Alloc as CAlloc;

/// `CMmapAlloc` implements `GlobalAlloc` and behaves like `std::alloc::System` except for the
/// followings.
///
/// - `CMmapAlloc` increases/decreases the caching byte size as allocate/deallocate heap memory.
/// - `CMmapAlloc` calls unix 'mmap(2)' to allocate heap memory.
pub use mouse_cache_alloc::MmapAlloc as CMmapAlloc;

/// `CVec` behaves like `std::vec::Vec` except for the followings.
///
/// - `CVec` does not implement methods to cost 'O(n)' CPU time on purpose.
/// - `CVec` uses [`CAlloc`] to allocate/deallocate heap memory.
///
/// [`CAlloc`]: struct.CAlloc.html
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CVec<T> {
    buffer: mouse_containers::Vec<T, CAlloc>,
}

impl<T> From<Vec<T>> for CVec<T> {
    fn from(vec: Vec<T>) -> Self {
        unsafe {
            let buffer = mouse_containers::Vec::<T, CAlloc>::from_vec_alloc(vec, CAlloc::default());
            let allocating_size = mouse_cache_alloc::allocating_size(buffer.as_ptr());
            mouse_cache_alloc::increase_cache_size(allocating_size);

            Self { buffer }
        }
    }
}

impl<T> From<&[T]> for CVec<T>
where
    T: Clone,
{
    fn from(vals: &[T]) -> Self {
        Self {
            buffer: mouse_containers::Vec::from_slice(vals, CAlloc::default()),
        }
    }
}

impl<T> CVec<T> {
    /// Creates a new empty instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let _cvec = CVec::<u8>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: mouse_containers::Vec::from(CAlloc::default()),
        }
    }
}

impl<T> AsRef<[T]> for CVec<T> {
    fn as_ref(&self) -> &[T] {
        self.buffer.as_ref()
    }
}

impl<T> AsMut<[T]> for CVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.buffer.as_mut()
    }
}

impl<T> Borrow<[T]> for CVec<T> {
    fn borrow(&self) -> &[T] {
        self.buffer.borrow()
    }
}

impl<T> BorrowMut<[T]> for CVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.buffer.borrow_mut()
    }
}

impl<T> Deref for CVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.buffer.deref()
    }
}

impl<T> DerefMut for CVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.deref_mut()
    }
}

impl<T, I> Index<I> for CVec<T>
where
    I: SliceIndex<[T], Output = T>,
{
    type Output = T;

    fn index(&self, i: I) -> &Self::Output {
        self.buffer.index(i)
    }
}

impl<T, I> IndexMut<I> for CVec<T>
where
    I: SliceIndex<[T], Output = T>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        self.buffer.index_mut(i)
    }
}

impl<'a, T> IntoIterator for &'a CVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut CVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.buffer).into_iter()
    }
}

impl<T> CVec<T> {
    /// Clones and appends all the elements in `vals` to the end of `self` .
    ///
    /// # Warnings
    ///
    /// This method calls `self.reserve(vals.len())` everytime.
    /// It makes the performance better to call [`reserve`] in advance to call this method twice
    /// or more than twice.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.extend_from_slice(&[0, 1, 2, 3]);
    /// assert_eq!(&[0, 1, 2, 3], cvec.as_ref());
    /// ```
    pub fn extend_from_slice(&mut self, vals: &[T])
    where
        T: Clone,
    {
        self.buffer.extend_from_slice(vals);
    }

    /// Appends `val` to the end of the buffer.
    ///
    /// # Warnings
    ///
    /// This method calls `self.reserve(1)` internally.
    /// It makes the performance better to call [`reserve`] in advance to call this method twice
    /// or more than twice.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.push(1);
    /// cvec.push(2);
    /// assert_eq!(&[1, 2], cvec.as_ref());
    /// ```
    pub fn push(&mut self, val: T) {
        self.buffer.push(val);
    }

    /// Removes the last element from `self` and returns it if any, or `None` .
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    ///
    /// assert_eq!(None, cvec.pop());
    ///
    /// cvec.push(1);
    /// cvec.push(2);
    /// assert_eq!(2, cvec.pop().unwrap());
    /// assert_eq!(1, cvec.pop().unwrap());
    /// assert_eq!(None, cvec.pop());
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.buffer.pop()
    }

    /// Returns the number of elements that `self` holds.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// for i in 0..10 {
    ///     assert_eq!(i, cvec.len());
    ///     cvec.push(0);
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Forces the length of `self` `new_len` .
    ///
    /// This is a low-level operation that maintains none of the normal invariants of the type.
    /// Normally changing the length of a vector is done using one of the safe operations instead,
    /// such as [`truncate`] or [`clear`] .
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to the capacity.
    /// - The elements at `old_len..new_len` must be initialized.
    ///
    /// [`truncate`]: #method.truncate
    /// [`clear`]: #method.clear
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.reserve(10);
    /// for i in 0..10 {
    ///     unsafe { cvec.set_len(i) };
    ///     assert_eq!(i, cvec.len());
    /// }
    /// ```
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.buffer.set_len(new_len);
    }

    /// Returns the number of elements that `self` can hold without allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// for i in 0..10 {
    ///     cvec.reserve(i);
    ///     assert!(i <= cvec.capacity());
    /// }
    /// ```
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Reserves capacity at least `additional` more elements to be inserted in `self` .
    ///
    /// This method does nothing if the capacity is already sufficient.
    /// After this method is called, the capacity will be greater than or equal to
    /// `self.len() + additional` .
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// for i in 0..10 {
    ///     cvec.reserve(i);
    ///     assert!(i <= cvec.capacity());
    /// }
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

    /// Returns `true` if `self` does not hold any element, or `false` .
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// assert_eq!(true, cvec.is_empty());
    ///
    /// cvec.push(0);
    /// assert_eq!(false, cvec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns a raw pointer to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.push(7);
    ///
    /// let ptr = cvec.as_ptr();
    /// let first = unsafe { *ptr };
    /// assert_eq!(7, first);
    /// ```
    pub fn as_ptr(&self) -> *const T {
        self.buffer.as_ptr()
    }

    /// Returns a mutable pointer to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.push(7);
    ///
    /// let ptr = cvec.as_mut_ptr();
    /// unsafe { *ptr += 1 };
    /// assert_eq!(8, cvec[0]);
    /// ```
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buffer.as_mut_ptr()
    }

    /// Shrinks the capacity to the length as much as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.reserve(1000);
    /// let old_capacity = cvec.capacity();
    /// assert!(1000 <= old_capacity);
    ///
    /// cvec.shrink_to_fit();
    /// assert!(cvec.capacity() <= old_capacity);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }

    /// Enshorten `self` , keeping the first `len` elements and drops the rest if `len` is less
    /// than the current length; otherwise does nothing.
    ///
    /// Note that this method does not have any effect on the allocated capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.extend_from_slice(&[0, 1, 2, 3, 4]);
    /// assert_eq!(5, cvec.len());
    ///
    /// cvec.truncate(7);
    /// assert_eq!(5, cvec.len());
    ///
    /// cvec.truncate(2);
    /// assert_eq!(2, cvec.len());
    /// assert_eq!(&[0, 1], cvec.as_ref());
    /// ```
    pub fn truncate(&mut self, len: usize) {
        self.buffer.truncate(len);
    }

    /// Clears `self` , removing all the elements.
    ///
    /// Note that this method does not have any effect on the allocated capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::CVec;
    ///
    /// let mut cvec = CVec::<u8>::new();
    /// cvec.extend_from_slice(&[0, 1, 2, 3, 4]);
    /// assert_eq!(5, cvec.len());
    ///
    /// cvec.clear();
    /// assert_eq!(0, cvec.len());
    /// ```
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
