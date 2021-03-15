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
use core::ops::Deref;
pub use crypto_hash::{CryptoHash, CryptoHasher};
pub use resource::{Resource, ResourceId, RESOURCE_ID_BUFFER_CAPACITY};
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
pub type CVec<T> = mouse_containers::Vec<T, CAlloc>;

/// `CVec` behaves like `std::vec::Vec` except for the followings.
///
/// - `CVec` does not implement methods to cost 'O(n)' CPU time on purpose.
/// - `CVec` uses [`CAlloc`] to allocate/deallocate heap memory.
///
/// [`CAlloc`]: struct.CAlloc.html
#[derive(Clone, Default)]
pub struct CVec_<T> {
    buffer: mouse_containers::Vec<T, CAlloc>,
}

impl<T> From<&[T]> for CVec_<T>
where
    T: Clone,
{
    fn from(vals: &[T]) -> Self {
        Self {
            buffer: mouse_containers::Vec::from_slice(vals, CAlloc::default()),
        }
    }
}

impl<T> Deref for CVec_<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.buffer.deref()
    }
}
