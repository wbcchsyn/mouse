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

//! # Logger
//!
//! Logger enable to use macros defined in log crate.
//! (See [log](https://crates.io/crates/log "log") for details.)
//!
//! ## Implementation
//!
//! Programmer should implement the followings.
//!
//! ### pub fn arguments
//!
//! `pub fn arguments(app: App<'static, 'static>) -> App<'static, 'static>`
//!
//! `arguments` adds the arguments for this module.
//! type `App` is defined in crate `clap` .
//! See [clap](https://crates.io/crates/clap "clap") for details.
//!
//! ### pub fn initializer
//!
//! `pub fn initializer(config: GlobalConfig) -> Result<impl ModuleInitializer, String>`
//!
//! On success, returns the implementation for ModuleInitializer, or the error message.
//!
//! ### stub implements ModuleInitializer
//!
//! The implementation for this module.
//!
//! `ModuleInitializer::init` should enable to use macros defined in `log` crate.
//! See [log](https://crates.io/crates/log "log") for details.

mod custom_logger;

pub use custom_logger::*;
