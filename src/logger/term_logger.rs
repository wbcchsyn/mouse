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

use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::result::Result;
use log::LevelFilter;
use simplelog::{TermLogger, TerminalMode};
use std::error::Error;

/// `Environment` implements `ModuleEnvironment` .
pub struct Environment {
    level: LevelFilter,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            level: LevelFilter::Warn,
        }
    }
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.arg(
            Arg::with_name("log_level")
                .possible_values(&["TRACE", "DEBUG", "INFO", "WARN", "ERROR"])
                .long("log-level")
                .default_value("WARN")
                .takes_value(true),
        )
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        match config.args().value_of("log_level").unwrap() {
            "TRACE" => self.level = LevelFilter::Trace,
            "DEBUG" => self.level = LevelFilter::Debug,
            "INFO" => self.level = LevelFilter::Info,
            "WARN" => self.level = LevelFilter::Warn,
            "Error" => self.level = LevelFilter::Error,
            arg => {
                let msg = format!("Bad parameter for '--log-level': {}", arg);
                return Err(Box::from(msg));
            }
        }

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        TermLogger::init(self.level, Default::default(), TerminalMode::Stdout).map_err(|e| {
            let msg = format!("Failed to open log: {}", e);
            Box::from(msg)
        })
    }
}
