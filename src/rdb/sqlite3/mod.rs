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

mod error;
mod stmt;

use crate::{Config, ModuleEnvironment};
use clap::App;
use std::os::raw::c_int;

pub use error::Error;
use stmt::Stmt;

// libsqlite3 error constants
// https://www.sqlite.org/draft/rescode.html
const SQLITE_OK: c_int = 0;
const SQLITE_DONE: c_int = 101;
const SQLITE_ROW: c_int = 100;

/// `Environment` implements `ModuleEnvironment` for this module.
#[derive(Default)]
pub struct Environment {}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app
    }

    unsafe fn check(&mut self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[allow(non_camel_case_types)]
enum sqlite3_stmt {}

#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_finalize(pstmt: *mut sqlite3_stmt) -> c_int;
}
