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

use crate::data_types::{CAcid, CMmapAlloc};
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::result::Result;
use mouse_containers::lru_hash_set::LruHashSet;
use std::collections::hash_map::RandomState;
use std::error::Error;

/// 64 MB.
const DEFAULT_SIZE_SOFT_LIMIT: &'static str = "67108864";

/// `Environment` implements `ModuleEnvironment` for this module.
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
        Ok(())
    }
}
