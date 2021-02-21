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

use super::Acid;
use crate::data_types::CAlloc;
use core::ops::Deref;
use counting_pointer::Asc;

/// `CAcid` is like `std::Arc<dyn 'static + Sync + Send + Acid>` except for the followings.
///
/// - `CAcid` does not support weak count for the performance.
/// - `CAcid` uses [`CAlloc`] to allocate/deallocate heap memory.
pub struct CAcid(Asc<dyn 'static + Sync + Send + Acid, CAlloc>);

impl<T> From<T> for CAcid
where
    T: 'static + Sync + Send + Acid,
{
    #[inline]
    fn from(val: T) -> Self {
        let asc = Asc::new(val, CAlloc::default());
        let (ptr, alloc) = Asc::into_raw_alloc(asc);
        let ptr = ptr as *const (dyn 'static + Sync + Send + Acid);
        let asc = unsafe { Asc::from_raw_alloc(ptr, alloc) };
        Self(asc)
    }
}

impl Deref for CAcid {
    type Target = dyn 'static + Sync + Send + Acid;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
