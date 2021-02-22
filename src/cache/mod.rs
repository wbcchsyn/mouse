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

use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::result::Result;
use std::error::Error;

/// 64 MB.
const DEFAULT_SIZE_SOFT_LIMIT: &'static str = "67108864";

/// `Environment` implements `ModuleEnvironment` for this module.
pub struct Environment {}

impl Default for Environment {
    fn default() -> Environment {
        Self {}
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

    unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
