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

use crate::{GlobalConfig, ModuleInitializer};
use clap::App;
use core::result::Result;

/// Adds the arguments for this module.
///
/// This function is stub so far.
/// Programmer should implement it.
pub fn arguments(_app: App<'static, 'static>) -> App<'static, 'static> {
    panic!("custom_logger::arguments is not implemented yet.");
}

/// On success, returns the implementation for ModuleInitializer, or the error message.
///
/// This function is stub so far.
/// Programmer should implement it.
pub fn initializer(_config: GlobalConfig) -> Result<Initializer, String> {
    panic!("custom_logger::initializer is not implemented yet.");
}

/// Implementation for ModuleInitializer.
///
/// `ModuleInitializer::init` should enable to use macros defined in `log` crate.
/// (See [log](https://crates.io/crates/log "log") for details.)
pub struct Initializer;

impl ModuleInitializer for Initializer {
    fn init(&self) -> Result<(), String> {
        panic!("custom_logger::Initializer::init is not implemented yet.");
    }
}
