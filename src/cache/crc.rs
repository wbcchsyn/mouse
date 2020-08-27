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

use super::CacheAlloc;
use core::alloc::{GlobalAlloc, Layout};
use core::any::Any;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

struct Bucket<T: ?Sized> {
    rc: AtomicUsize, // Reference count
    elm: T,
}

/// A thread safe reference counting pointer.
/// 'Crc' stands for 'Cache Reference Counted.'
///
/// `Crc` behaves like `std::sync::Arc` except for
///
/// - `Crc` supports only strong count, not weak count.
/// - `Crc` increase/decrease cache memory usage when allocating/deallocating
///   heap memory.
/// - `Crc` itself doesn't have type parameter of inner type. The caller
///   should access to `dyn Any` and cast it.
///
/// # Warnings
///
/// `Crc` doesn't know how inner element uses heap memory.
/// It should increase/decrease cache memory usage when
/// allocating/deallocating by itself.
pub struct Crc {
    ptr: NonNull<Bucket<dyn Any>>,
    layout: Layout,
}
