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
    sqlite3, sqlite3_column_count, sqlite3_finalize, sqlite3_prepare_v2, sqlite3_stmt, Error,
    SQLITE_TOOBIG,
};
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::ptr;
use std::os::raw::{c_char, c_int};

/// Wrapper of C [`sqlite3_stmt`] .
///
/// [`sqlite3_stmt`]: https://www.sqlite.org/c3ref/stmt.html
pub struct Stmt<'a> {
    raw: *mut sqlite3_stmt,
    column_count: c_int,
    is_row: bool,
    _con: PhantomData<&'a mut sqlite3>,
    _sql: PhantomData<&'a str>,
}

impl Drop for Stmt<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.raw) };
    }
}

impl<'a> Stmt<'a> {
    /// Creates a new instance.
    pub fn new(sql: &'a str, connection: &'a mut sqlite3) -> Result<Self, Error> {
        let con = connection as *mut sqlite3;
        let zsql = sql.as_ptr() as *const c_char;
        let nbytes = c_int::try_from(sql.len()).or(Err(Error::new(SQLITE_TOOBIG)))?;
        let mut raw: *mut sqlite3_stmt = ptr::null_mut();
        let mut pztail: *const c_char = ptr::null();

        let code = unsafe { sqlite3_prepare_v2(con, zsql, nbytes, &mut raw, &mut pztail) };
        match Error::new(code) {
            Error::OK => {
                let column_count = unsafe { sqlite3_column_count(raw) };
                Ok(Stmt {
                    raw,
                    column_count,
                    is_row: false,
                    _con: PhantomData,
                    _sql: PhantomData,
                })
            }
            e => Err(e),
        }
    }
}
