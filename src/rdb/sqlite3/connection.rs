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

use super::{
    sqlite3, sqlite3_close, sqlite3_open_v2, Error, Stmt, SQLITE_OPEN_MEMORY, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READWRITE,
};
use core::convert::TryFrom;
use core::ptr;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;

/// New type of `&'static str` , which is compared by the address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Sql(*const u8);

#[cfg(test)]
mod sql_tests {
    use super::*;

    #[test]
    fn equality() {
        const A: &'static str = "aa";
        const B: &'static str = "ab";
        const C: &'static str = "a";

        assert_ne!(Sql(A.as_ptr()), Sql(B.as_ptr()));
        assert_ne!(Sql(A.as_ptr()), Sql(C.as_ptr()));
    }
}

/// Wrapper of C struct [`sqlite3`]
///
/// [`sqlite3`]: https://www.sqlite.org/c3ref/sqlite3.html
pub struct Connection {
    raw: *mut sqlite3,
    stmts: HashMap<Sql, Stmt<'static>>,
    is_transaction: bool,
}

unsafe impl Send for Connection {}

impl Drop for Connection {
    #[inline]
    fn drop(&mut self) {
        self.stmts.clear(); // All the Stmt instances must be finalized before close.
        unsafe { sqlite3_close(self.raw) };
    }
}

impl TryFrom<&Path> for Connection {
    type Error = Box<dyn std::error::Error>;

    #[inline]
    fn try_from(filename: &Path) -> Result<Self, Self::Error> {
        let filename = CString::new(filename.to_string_lossy().as_bytes()).map_err(Box::new)?;
        let mut raw: *mut sqlite3 = ptr::null_mut();
        const FLAGS: c_int = SQLITE_OPEN_READWRITE | SQLITE_OPEN_MEMORY | SQLITE_OPEN_NOMUTEX;
        const ZVFS: *const c_char = ptr::null();

        let code = unsafe { sqlite3_open_v2(filename.as_ptr(), &mut raw, FLAGS, ZVFS) };
        match Error::new(code) {
            Error::OK => Ok(Self {
                raw,
                stmts: Default::default(),
                is_transaction: false,
            }),
            e => Err(Box::new(e)),
        }
    }
}

impl Connection {
    /// Opens in-memory database and returns a new instance.
    #[inline]
    pub fn open_memory_db() -> Result<Self, Error> {
        let filename: *const c_char = "memory_db".as_ptr() as *const c_char;
        let mut raw: *mut sqlite3 = ptr::null_mut();
        const FLAGS: c_int = SQLITE_OPEN_MEMORY | SQLITE_OPEN_READWRITE | SQLITE_OPEN_NOMUTEX;
        const ZVFS: *const c_char = ptr::null();

        let code = unsafe { sqlite3_open_v2(filename, &mut raw, FLAGS, ZVFS) };
        match Error::new(code) {
            Error::OK => Ok(Self {
                raw,
                stmts: Default::default(),
                is_transaction: false,
            }),
            e => Err(e),
        }
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[test]
    fn memory_db_constructor() {
        assert_eq!(true, Connection::open_memory_db().is_ok());
    }
}
