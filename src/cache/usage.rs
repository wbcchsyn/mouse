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
