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

//! Implementation for `logger` module.
//!
//! Log will be written in stderr.

extern crate simplelog;

use crate::{GlobalConfig, ModuleInitializer};
use clap::{App, Arg};
use core::result::Result;
use log::LevelFilter;
use simplelog::{Config, TermLogger, TerminalMode};

/// Adds the arguments for this module.
pub fn arguments(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("log_level")
            .help("DEBUG|INFO|WARN|ERROR (Default is WARN.)")
            .long("log_level")
            .takes_value(true),
    )
}

/// Sanitizes the argument and build Initializer.
/// On error, returns the error message.
pub fn initializer(config: GlobalConfig) -> Result<Initializer, String> {
    let level = if let Some(level) = config.args().value_of("log_level") {
        match level {
            "TRACE" => LevelFilter::Trace,
            "DEBUG" => LevelFilter::Debug,
            "INFO" => LevelFilter::Info,
            "WARN" => LevelFilter::Warn,
            "ERROR" => LevelFilter::Error,
            _ => return Err(format!("Unknown log level {}.", level)),
        }
    } else {
        LevelFilter::Warn
    };

    Ok(Initializer { level })
}

/// Implementation for ModuleInitializer.
pub struct Initializer {
    level: LevelFilter,
}

impl ModuleInitializer for Initializer {
    fn init(&self) -> Result<(), String> {
        TermLogger::init(self.level, Config::default(), TerminalMode::Stderr)
            .or_else(|e| Err(e.to_string()))
    }
}
