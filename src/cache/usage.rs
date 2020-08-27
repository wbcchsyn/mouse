// Copyright 2020 Shin Yoshida
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

use core::sync::atomic::{AtomicUsize, Ordering};
use std::os::raw::c_void;

static USAGE: AtomicUsize = AtomicUsize::new(0);

/// Returns how many bytes cache is using.
///
/// # Warnings
///
/// This function doesn't acquire any lock for the performance.
/// The result is not always the latest value.
pub fn usage() -> usize {
    USAGE.load(Ordering::Relaxed)
}

/// Increases the memory usage for cache by `byte_size`, returning
/// the new value.
///
/// # Warnings
///
/// This function doesn't acquire any lock for the performance.
/// The result is not always the latest value.
pub fn add_usage(byte_size: usize) -> usize {
    USAGE.fetch_add(byte_size, Ordering::Relaxed) + byte_size
}

/// Decreases the memory usage for cache by `byte_size`, returning
/// the new value.
///
/// # Warnings
///
/// This function doesn't acquire any lock for the performance.
/// The result is not always the latest value.
pub fn sub_usage(byte_size: usize) -> usize {
    USAGE.fetch_sub(byte_size, Ordering::Relaxed) + byte_size
}

/// Returns size of memory allocated from heap.
///
/// Argument `ptr` must be `std::alloc::alloc` returned, and
/// must not be deallocated yet.
///
/// # Safety
///
/// The behavior is undefined if `ptr` doesn't satisfy the
/// requirements.
///
/// # Warnings
///
/// This function works both in Linux `dmalloc` and `jemalloc`,
/// however, it is based on `malloc_usable_size` which is not defined
/// in POSIX.
///
/// The behavior is undefined if ptr is null.
pub unsafe fn allocation_size<T>(ptr: *const T) -> usize {
    // `malloc_usable_size` works even if ptr is null, however,
    // it should not be null for future extension and for performance.
    debug_assert_eq!(false, ptr.is_null());

    malloc_usable_size(ptr as *const c_void)
}

extern "C" {
    /// Returns size of memory allocated from heap.
    ///
    /// Argument `ptr` must be `std::alloc::alloc` returned and
    /// must not be deallocated yet, or null.
    ///
    /// If `ptr` is null, always returns 0.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `ptr` doesn't satisfy the
    /// requirements.
    ///
    /// # Warnings
    ///
    /// Both Linux `dmalloc` and `jemalloc`  implemnets this function,
    /// however, it is not defined in POSIX.
    /// For example, `tcmalloc` names `tc_malloc_size` the same function.
    fn malloc_usable_size(ptr: *const c_void) -> usize;
}
