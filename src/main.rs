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
extern crate errno;

mod logger;

use clap::{App, ArgMatches};
use core::result::Result;
use std::os::raw::c_int;

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

/// Starts logging, initializes each module, and waits for the signal.
///
/// If some error is occurred before starting log, returns the error message;
/// otherwise returns Ok.
fn run(config: GlobalConfig) -> Result<(), String> {
    // Initialize logger first for other ModuleInitializers to enable to log.
    let logger_ = logger::initializer(config)?;
    logger_.init()?;

    warn!("Mouse is starting...");

    // Initialize the other modules here.

    warn!("Mouse is started.");

    unsafe {
        if sigwait_() != 0 {
            error!("{}", errno::errno());
            return Ok(());
        }
    }

    warn!("Mouse is stopping...");
    Ok(())
}

fn main() {
    let config = parse_argument();

    if let Err(e) = run(config) {
        eprintln!("{}", e);
    }

    warn!("Mouse stopped.");
}

#[link(name = "mouse_signal")]
extern "C" {
    /// Waits for signal SIGHUP, SIGTERM, and SIGINT.
    ///
    /// On success returns 0; otherwise sets errno and returns 1.
    fn sigwait_() -> c_int;
}
