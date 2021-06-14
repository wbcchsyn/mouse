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

mod connection;
mod error;
mod stmt;

use super::Session;
use crate::{Config, ModuleEnvironment};
use clap::{App, Arg};
use core::cell::Cell;
use core::convert::TryFrom;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;

use connection::Connection;
pub use error::Error;
use stmt::Stmt;

// libsqlite3 error constants
// https://www.sqlite.org/draft/rescode.html
const SQLITE_OK: c_int = 0;
const SQLITE_TOOBIG: c_int = 18;
const SQLITE_RANGE: c_int = 25;
const SQLITE_DONE: c_int = 101;
const SQLITE_ROW: c_int = 100;

// Constants for column type
// https://www.sqlite.org/draft/c3ref/c_blob.html
const SQLITE_INTEGER: c_int = 1;
const SQLITE_BLOB: c_int = 4;
const SQLITE_NULL: c_int = 5;

// Constants for sqlite3_open_v2()
// https://www.sqlite.org/draft/c3ref/c_open_autoproxy.html
const SQLITE_OPEN_READWRITE: c_int = 0x00000002;
const SQLITE_OPEN_MEMORY: c_int = 0x00000080;
const SQLITE_OPEN_NOMUTEX: c_int = 0x00008000;

/// `Environment` implements `ModuleEnvironment` for this module.
pub struct Environment {
    data_path: PathBuf,
    session_owner: (Mutex<Option<ThreadId>>, Condvar),
    connection: Cell<Connection>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            data_path: PathBuf::default(),
            session_owner: Default::default(),
            connection: Cell::new(Connection::open_memory_db().unwrap()),
        }
    }
}

impl ModuleEnvironment for Environment {
    fn args(app: App<'static, 'static>) -> App<'static, 'static> {
        app.arg(
            Arg::with_name("PATH_TO_RDB_DATA_DIR")
                .help("Path to the RDB database directory.")
                .long("--rdb-data-path")
                .required(true)
                .takes_value(true),
        )
    }

    unsafe fn check(&mut self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let data_path = config.args().value_of("PATH_TO_RDB_DATA_DIR").unwrap();
        self.data_path = PathBuf::from(data_path);

        Ok(())
    }

    unsafe fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.connection = Cell::new(Connection::try_from(self.data_path.as_ref())?);

        Ok(())
    }
}

#[allow(non_camel_case_types)]
enum sqlite3_stmt {}

#[allow(non_camel_case_types)]
pub enum sqlite3 {}

struct Sqlite3Session<'a> {
    env: &'a Environment,
    con: &'a mut Connection,
    is_transaction_: bool,
}

impl Drop for Sqlite3Session<'_> {
    fn drop(&mut self) {
        // Rollback for just in case.
        // do_rollback() returns an error if transaction is not started.
        // Ignore the error.
        let _ = self.do_rollback();

        let (mtx, cond) = &self.env.session_owner;
        let mut guard = mtx.lock().unwrap();
        *guard = None;
        cond.notify_one();
    }
}

impl Session for Sqlite3Session<'_> {
    fn is_transaction(&self) -> bool {
        self.is_transaction_
    }

    fn begin_transaction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(false, self.is_transaction_);
        // The compiler can't assume the type to use map_err().
        match self.do_begin_transaction() {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn commit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(true, self.is_transaction_);
        // The compiler can't assume the type to use map_err().
        match self.do_commit() {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn rollback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(true, self.is_transaction_);
        // The compiler can't assume the type to use map_err().
        match self.do_rollback() {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Sqlite3Session<'_> {
    fn do_begin_transaction(&mut self) -> Result<(), Error> {
        const SQL: &'static str = "BEGIN";
        let stmt = self.con.stmt(SQL)?;
        stmt.step()?;

        self.is_transaction_ = true;
        Ok(())
    }

    fn do_commit(&mut self) -> Result<(), Error> {
        const SQL: &'static str = "COMMIT";
        let stmt = self.con.stmt(SQL)?;
        stmt.step()?;

        self.is_transaction_ = false;
        Ok(())
    }

    fn do_rollback(&mut self) -> Result<(), Error> {
        const SQL: &'static str = "ROLLBACK";
        let stmt = self.con.stmt(SQL)?;
        stmt.step()?;

        self.is_transaction_ = false;
        Ok(())
    }
}

#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_open_v2(
        filename: *const c_char,
        ppdb: *mut *mut sqlite3,
        flags: c_int,
        zvfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(pdb: *mut sqlite3) -> c_int;

    fn sqlite3_prepare_v2(
        pdb: *mut sqlite3,
        zsql: *const c_char,
        nbyte: c_int,
        ppstmt: *mut *mut sqlite3_stmt,
        pztail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_finalize(pstmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_column_count(pstmt: *mut sqlite3_stmt) -> c_int;

    fn sqlite3_reset(pstmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_clear_bindings(pstmt: *mut sqlite3_stmt) -> c_int;

    fn sqlite3_step(pstmt: *mut sqlite3_stmt) -> c_int;

    fn sqlite3_bind_int64(pstmt: *mut sqlite3_stmt, index: c_int, val: i64) -> c_int;
    fn sqlite3_bind_blob(
        pstmt: *mut sqlite3_stmt,
        index: c_int,
        pval: *const c_void,
        vlen: c_int,
        destructor: *const c_void,
    ) -> c_int;
    fn sqlite3_bind_null(pstmt: *mut sqlite3_stmt, index: c_int) -> c_int;

    fn sqlite3_column_type(pstmt: *mut sqlite3_stmt, icol: c_int) -> c_int;
    fn sqlite3_column_int64(pstmt: *mut sqlite3_stmt, icol: c_int) -> i64;
    fn sqlite3_column_blob(pstmt: *mut sqlite3_stmt, icol: c_int) -> *const c_void;
    fn sqlite3_column_bytes(pstmt: *mut sqlite3_stmt, icol: c_int) -> c_int;
}
