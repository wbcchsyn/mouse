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

//! `cache` provides cache system for mouse.
//! `cache` may depend on module `data_types` , but is independent from other modules.

// //////////////////////////////////////
//
// # Warnings
//
// 'Mouse' is using 'mouse_containers::lru_hash_set::LruHashSet' for the cache system.
// Don't leak 'mouse_containers::lru_hash_set::Entry' to help a dead lock.
//
// //////////////////////////////////////

use crate::data_types::{Acid, CAcid, CMmapAlloc, Id, Resource};
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::any::TypeId;
use core::mem::size_of;
use core::result::Result;
use mouse_containers::lru_hash_set::LruHashSet;
use spin_sync::Mutex8;
use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::error::Error;

/// 64 MB.
const DEFAULT_SIZE_SOFT_LIMIT: &'static str = "67108864";

/// `Environment` implements `ModuleEnvironment` for this module.
///
/// # Arguments
///
/// `Environment` requests the following arguments.
///
/// - --cache-size-soft-limit
///
/// # Default
///
/// The `Default` implementation assumes the following arguments.
///
/// - --cache-size-soft-limit: 67108864 (= 64 MB)
pub struct Environment {
    size_soft_limit: usize,
    cache: LruHashSet<CAcid, CMmapAlloc, RandomState>,
}

impl Default for Environment {
    fn default() -> Environment {
        Self {
            size_soft_limit: DEFAULT_SIZE_SOFT_LIMIT.parse().unwrap(),
            cache: LruHashSet::new(CMmapAlloc::default(), RandomState::new()),
        }
    }
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.arg(
            Arg::with_name("cache_size_soft_limit")
                .help(
                    "The soft limit of cache byte size.
The LRU cache is expired when the total cache size exceeds this value.",
                )
                .long("--cache-size-soft-limit")
                .default_value(DEFAULT_SIZE_SOFT_LIMIT)
                .takes_value(true),
        )
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let size_soft_limit = config.args().value_of("cache_size_soft_limit").unwrap();
        self.size_soft_limit = size_soft_limit.parse().map_err(|e| {
            let msg = format!("Failed to parse '--cache-size-soft-limit': {}", e);
            Box::<dyn Error>::from(msg)
        })?;

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        // Use about 1/128 bytes of '--cache-size-soft-limit' for bucket chain.
        // 8 buckets consumes '8 * size_of::<raw pointer>() + 1 * size_of::<Mutex8>()' bytes.
        let bucket8_size = 8 * size_of::<*mut u8>() + size_of::<Mutex8>();
        let chain_len = self.size_soft_limit / 128 * 8 / bucket8_size;

        // 'chain_len' must be greater than 0 (excluding 0), and (I think) it should not be a round
        // value.
        let chain_len = chain_len + 1;
        self.cache.init(chain_len);

        Ok(())
    }
}

/// `NotFound` represents the data is not found in KVS.
///
/// `NotFound` implements [`Acid`] , but all the methods except for `id` and `type_id` causes a
/// panic.
///
/// [`Acid`]: crate::data_types::Acid
struct NotFound(Id);

impl From<Id> for NotFound {
    fn from(id: Id) -> Self {
        Self(id)
    }
}

impl Acid for NotFound {
    fn id(&self) -> &Id {
        &self.0
    }

    fn intrinsic(&self) -> Cow<[u8]> {
        panic!("Method 'NotFound.intrinsic' is called.");
    }

    fn extrinsic(&self) -> Cow<[u8]> {
        panic!("Method 'NotFound.extrinsic' is called.");
    }

    fn parent_count(&self) -> usize {
        panic!("Method 'NotFound.parent_count' is called.");
    }

    fn parent(&self, _index: usize) -> Option<Id> {
        panic!("Method 'NotFound.parent' is called.");
    }

    fn resource_count(&self) -> usize {
        panic!("Method 'NotFound.resource_count' is called");
    }

    fn resource(&self, _index: usize) -> Option<Resource> {
        panic!("Method 'NotFound.resource' is called.");
    }

    fn is_traceable(&self) -> bool {
        panic!("Method 'NotFound.is_traceable' is called.");
    }

    fn set_traceable(&self) -> bool {
        panic!("Method 'Notfound.set_traceable' is called.");
    }

    fn is_invalid(&self) -> bool {
        panic!("Method 'NotFound.is_invalid' is called.");
    }

    fn invalid_reason(&self) -> Option<&dyn Error> {
        panic!("Method 'NotFound.invalid_reason' is called.");
    }

    unsafe fn merge(&self, _other: &dyn Acid) -> bool {
        panic!("Method 'NotFound.merge' is called.");
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// `CacheFindResult` is return value for function [`find`] .
///
/// [`find`]: self::find
pub enum CacheFindResult {
    /// The element is stored in DataBase and cached.
    Hit(CAcid),
    /// The cache does not know about the element at all.
    Lost,
    /// The last DataBase query found no such data is stored in DataBase.
    Fault,
}

fn is_not_found(val: &CAcid) -> bool {
    val.downcast::<NotFound>().is_some()
}

/// Returns the byte size that the cache system is using.
pub fn cache_using_byte_size() -> usize {
    mouse_cache_alloc::cache_size()
}

/// Increases the cache using size by `bytes` and returns the new using size.
pub fn increase_cache_using_size(bytes: usize) -> usize {
    mouse_cache_alloc::increase_cache_size(bytes)
}

/// Decreases the cache using size by `bytes` and returns the new using size.
pub fn decrease_cache_using_size(bytes: usize) -> usize {
    mouse_cache_alloc::decrease_cache_size(bytes)
}

/// Finds cache whose id equals to `id` and returns the result.
///
/// The found cache element will be regarded as the 'Most Recently Used (MRU)'.
pub fn find(id: &Id, environment: &Environment) -> CacheFindResult {
    match unsafe { environment.cache.get(id) } {
        None => CacheFindResult::Lost,
        Some(entry) => {
            entry.to_mru();

            if is_not_found(&*entry) {
                CacheFindResult::Fault
            } else {
                CacheFindResult::Hit(entry.clone())
            }
        }
    }
}

/// Inserts `val` into the cache if not cached yet; otherwise merges the information into the
/// current cache element and drops `val` .
///
/// Inserted `val` or current cache element will be regarded as the 'Most Recently Used (MRU)'
/// anyway.
pub fn insert(val: CAcid, environment: &Environment) {
    debug_assert_eq!(false, is_not_found(&val));

    // Insert into the cache.
    let op = |element: &mut CAcid, val: CAcid| {
        if is_not_found(element) {
            // If element represents 'Not found', replace it.
            *element = val;
        } else {
            // Merge the information.
            unsafe { element.merge(&*val) };
        }
    };
    match unsafe { environment.cache.insert_with(val, op) } {
        (Some(_), entry) => {
            // The same id element exists.
            // Update the LRU order.
            entry.to_mru();
        }
        _ => {
            // `val` is inserted newly.
            // Do nothing because it is added as an MRU element.
        }
    }

    // Expire the LRU cache if the caching size exceeds the soft limit.
    while environment.size_soft_limit < cache_using_byte_size() {
        if !unsafe { environment.cache.expire() } {
            break;
        }
    }
}

/// Caches that the DataBase query failed to find the data with `id` .
pub fn not_found(id: Id, environment: &Environment) {
    let val = CAcid::from(NotFound::from(id));

    // Do nothing if already cached.
    // (Do not update the LRU order.)
    let op = |_element: &mut CAcid, _: CAcid| {};
    match unsafe { environment.cache.insert_with(val, op) } {
        (None, entry) => {
            // 'val' is inserted newly.
            // The cache size could be enlarged.

            // Make sure to drop 'entry' to help a dead lock.
            drop(entry);

            // Expire the LRU cache if the caching size exceeds the soft limit.
            while environment.size_soft_limit < cache_using_byte_size() {
                if !unsafe { environment.cache.expire() } {
                    break;
                }
            }
        }
        _ => {
            // Nothing is changed.
        }
    }
}

/// Expires the 'Least Recently Used (LRU)' cache element and returns `true` if something is
/// cached; otherwise does nothing and returns `false` .
///
/// # Warnings
///
/// Cache memory is used for the following things.
/// The cache memory usage will not be 0 even if it is empty.
///
/// - Meta data for the cache system.
/// - Reserving memories for next cache.
/// - Cache elements that another thread is using.
///   (The cache element is really freed if it is expired and if all the threads finished to using
///   it.)
pub fn expire(environment: &Environment) -> bool {
    unsafe { environment.cache.expire() }
}

/// `CacheState` is return value for function [`is_cached`] .
///
/// [`is_cached`]: self::is_cached
pub enum CacheState {
    /// The element is stored in the DataBase and cached.
    Cached,
    /// The cache does not know about the element at all.
    Lost,
    /// The last DataBase query found no such data was stored in the DataBase.
    Fault,
}

/// Checks how the element with `id` is cached.
///
/// If the element is cached (either `Cached` or `Fault` ,) the cache entry will be regarded as
/// the 'Most Recently Used (MRU.)'
pub fn is_cached(id: &Id, environment: &Environment) -> CacheState {
    match unsafe { environment.cache.get(id) } {
        None => CacheState::Lost,
        Some(entry) => {
            entry.to_mru();

            if is_not_found(&*entry) {
                CacheState::Fault
            } else {
                CacheState::Cached
            }
        }
    }
}
