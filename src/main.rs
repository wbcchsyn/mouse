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

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod logger;

use clap::{App, ArgMatches};
use core::result::Result;

pub struct GlobalConfig {
    args_: ArgMatches<'static>,
}

impl GlobalConfig {
    /// Accessor to the arguments.
    pub fn args(&self) -> &ArgMatches<'static> {
        &self.args_
    }
}

/// Parses the arguments and returns GlobalConfig.
fn parse_argument() -> GlobalConfig {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!());

    let app = logger::arguments(app);

    GlobalConfig {
        args_: app.get_matches(),
    }
}

/// Trait to initialize each module.
///
/// The implementation for each module is created after parsing the arguments.
/// The constructor should sanitize the arguments. Then, method `init` is called
/// for the all instances. `init` should initialize the module if necessary.
/// Finally the instances are dropped before process exits. `Drop` trait should be
/// implemented if any cleanup process is.
pub trait ModuleInitializer {
    /// Initialize the module.
    ///
    /// Return the error message on error.
    fn init(&self) -> Result<(), String>;
}

fn run(config: GlobalConfig) {
    // Initialize logger first for other ModuleInitializers to enable to log.
    let logger_ = match logger::initializer(config) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    if let Err(e) = logger_.init() {
        eprintln!("{}", e);
        return;
    }
}

fn main() {
    let config = parse_argument();
    run(config);
}
