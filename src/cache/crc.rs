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

const ALLOC: CacheAlloc = CacheAlloc::new();
impl Crc {
    /// Creates a new instance.
    ///
    /// # Panics
    ///
    /// Panics if failed to allocate heap memory.
    pub fn new<T: 'static>(elm: T) -> Self {
        let layout = Layout::new::<Bucket<T>>();

        let bucket = Bucket {
            rc: AtomicUsize::new(1),
            elm,
        };

        unsafe {
            let ptr = ALLOC.alloc(layout) as *mut Bucket<T>;
            if ptr.is_null() {
                panic!("Failed to allocate heap memory.");
            }

            core::ptr::write(ptr, bucket);
            Crc {
                ptr: NonNull::new_unchecked(ptr),
                layout,
            }
        }
    }
}

impl Drop for Crc {
    fn drop(&mut self) {
        unsafe {
            // Decrease the reference count.
            let bucket = self.ptr.as_mut();
            let rc = bucket.rc.fetch_sub(1, Ordering::Release);

            // Drop and dealloc if this is the last reference.
            if rc == 1 {
                core::ptr::drop_in_place(&mut bucket.elm as *mut dyn Any);
                ALLOC.dealloc(self.ptr.as_ptr() as *mut u8, self.layout);
            }
        }
    }
}

impl Clone for Crc {
    fn clone(&self) -> Self {
        // Increase the reference count.
        unsafe {
            let bucket = self.ptr.as_ref();
            bucket.rc.fetch_add(1, Ordering::Acquire);

            Self {
                ptr: self.ptr,
                layout: self.layout,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::usage;

    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct Foo;
    impl Drop for Foo {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    #[ignore]
    fn constructor() {
        assert_eq!(0, usage());
        DROP_COUNT.store(0, Ordering::Relaxed);

        {
            let _crc = Crc::new(Foo);
            assert_ne!(0, usage());
            assert_eq!(0, DROP_COUNT.load(Ordering::Relaxed));
        }

        assert_eq!(0, usage());
        assert_eq!(1, DROP_COUNT.load(Ordering::Relaxed));
    }

    #[test]
    #[ignore]
    fn clone() {
        assert_eq!(0, usage());
        DROP_COUNT.store(0, Ordering::Relaxed);

        {
            let crc = Crc::new(Foo);
            let u = usage();
            assert_ne!(0, u);
            assert_eq!(0, DROP_COUNT.load(Ordering::Relaxed));

            {
                let _crc = crc.clone();
                assert_eq!(u, usage());
                assert_eq!(0, DROP_COUNT.load(Ordering::Relaxed));
            }

            assert_eq!(u, usage());
            assert_eq!(0, DROP_COUNT.load(Ordering::Relaxed));
        }

        assert_eq!(0, usage());
        assert_eq!(1, DROP_COUNT.load(Ordering::Relaxed));
    }
}
