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

use super::{add_usage, allocation_size, sub_usage};
use core::alloc::{GlobalAlloc, Layout};

/// Implements `std::alloc::GlobalAlloc` and behaves like `std::alloc::System`
/// except for increasing/decreasing cache memory usage when function
/// `alloc`/`dealloc` is called.
#[derive(Debug, Clone, Copy)]
pub struct CacheAlloc;

impl CacheAlloc {
    /// Create a new instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for CacheAlloc {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for CacheAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = std::alloc::alloc(layout);

        if !ptr.is_null() {
            add_usage(allocation_size(ptr));
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() {
            sub_usage(allocation_size(ptr));
        }

        std::alloc::dealloc(ptr, layout);
    }
}
