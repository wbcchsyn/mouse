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

use super::{sqlite3, sqlite3_finalize, sqlite3_stmt};
use core::marker::PhantomData;
use std::os::raw::c_int;

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
