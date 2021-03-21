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
use std::error::Error;
use std::path::PathBuf;

/// `Environment` implements `ModuleEnvironment` for this module.
#[derive(Default)]
pub struct Environment {
    db_path: PathBuf,
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.args(&[Arg::with_name("PATH_TO_KVS_DB_DIR")
            .help("Path to the KVS Database directory.")
            .long("--kvs-db-path")
            .required(true)
            .takes_value(true)])
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn Error>> {
        let db_path = config.args().value_of("PATH_TO_KVS_DB_DIR").unwrap();
        self.db_path = PathBuf::from(db_path);

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
