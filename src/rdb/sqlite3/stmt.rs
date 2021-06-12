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
    sqlite3, sqlite3_bind_blob, sqlite3_bind_int64, sqlite3_bind_null, sqlite3_clear_bindings,
    sqlite3_column_count, sqlite3_column_int64, sqlite3_column_type, sqlite3_finalize,
    sqlite3_prepare_v2, sqlite3_reset, sqlite3_step, sqlite3_stmt, Error, SQLITE_INTEGER,
    SQLITE_NULL, SQLITE_RANGE, SQLITE_TOOBIG,
};
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::ptr;
use std::os::raw::{c_char, c_int, c_void};

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

impl Stmt<'_> {
    /// Calls C function [`sqlite3_reset`] to clear the previous result.
    ///
    /// This method is called automatically if necessary, so the user will rarely call this method.
    /// Note this method does not change the binding parameters at all.
    ///
    /// [`sqlite3_reset`]: https://www.sqlite.org/c3ref/reset.html
    #[inline]
    pub fn reset(&mut self) {
        unsafe { sqlite3_reset(self.raw) };
        self.is_row = false;
    }

    /// Calls C function [`sqlite3_reset`] and [`sqlite3_clear_bindings`] to reset all the
    /// parameters.
    ///
    /// Because the document of [`sqlite3_clear_bindings`] is ambiguous, this method calls
    /// [`sqlite3_reset`] at the same time.
    ///
    /// [`sqlite3_reset`]: https://www.sqlite.org/c3ref/reset.html
    /// [`sqlite3_clear_bindings`]: https://www.sqlite.org/c3ref/clear_bindings.html
    #[inline]
    pub fn clear(&mut self) {
        self.reset();
        unsafe { sqlite3_clear_bindings(self.raw) };
    }

    /// Calls C function [`sqlite3_step`] and returns whether the SQL statement returns any
    /// data to be fetched.
    ///
    /// Returns `true` if the SQL statement being executed returns any data (i.e. [`sqlite3_step`]
    /// returned `SQLITE_ROW`.)
    ///
    /// Calls [`reset`] and returns `false` if the SQL statement has finished (i.e.
    /// [`sqlite3_step`] returned `SQLITE_DONE` . Then no data was returned.)
    ///
    /// Otherwise, i.e. [`sqlite3_step`] failed, calls [`reset`] and returns `Err` .
    ///
    /// [`reset`]: #method.reset
    /// [`sqlite3_step`]: https://www.sqlite.org/c3ref/step.html
    #[inline]
    pub fn step(&mut self) -> Result<bool, Error> {
        let code = unsafe { sqlite3_step(self.raw) };
        match Error::new(code) {
            Error::DONE => {
                self.reset();
                Ok(false)
            }
            Error::ROW => {
                self.is_row = true;
                Ok(true)
            }
            e => {
                self.reset();
                Err(e)
            }
        }
    }

    /// Wrapper of C function [`sqlite3_bind_int64`] .
    ///
    /// Calls method [`reset`] if necessary, and calls [`sqlite3_bind_int64`] .
    /// Note that `index` starts at 1, not 0.
    ///
    /// [`reset`]: #method.reset
    /// [`step`]: #method.step
    /// [`sqlite3_bind_int64`]: https://www.sqlite.org/c3ref/bind_blob.html
    /// [`sqlite3_reset`]: https://www.sqlite.org/c3ref/reset.html
    /// [`sqlite3_step`]: https://www.sqlite.org/c3ref/step.html
    #[inline]
    pub fn bind_int(&mut self, index: usize, val: i64) -> Result<(), Error> {
        // self.reset() was not called after self.step() returns true.
        if self.is_row {
            self.reset();
        }

        let index = c_int::try_from(index).or(Err(Error::new(SQLITE_RANGE)))?;
        let code = unsafe { sqlite3_bind_int64(self.raw, index, val) };
        match Error::new(code) {
            Error::OK => Ok(()),
            e => Err(e),
        }
    }

    /// Wrapper of C function [`sqlite3_bind_blob`] .
    ///
    /// Calls method [`reset`] if necessary, and calls [`sqlite3_bind_blob`] .
    /// Note that `index` starts at 1, not 0.
    ///
    /// [`reset`]: #method.reset
    /// [`step`]: #method.step
    /// [`sqlite3_bind_blob`]: https://www.sqlite.org/c3ref/bind_blob.html
    /// [`sqlite3_reset`]: https://www.sqlite.org/c3ref/reset.html
    /// [`sqlite3_step`]: https://www.sqlite.org/c3ref/step.html
    #[inline]
    pub fn bind_blob<'a, 'b>(&'a mut self, index: usize, val: &'b [u8]) -> Result<(), Error>
    where
        'b: 'a,
    {
        // self.reset() was not called after self.step() returns true.
        if self.is_row {
            self.reset();
        }

        let index = c_int::try_from(index).or(Err(Error::new(SQLITE_RANGE)))?;
        let ptr = val.as_ptr() as *const c_void;
        let len = c_int::try_from(val.len()).or(Err(Error::new(SQLITE_TOOBIG)))?;
        const DESTRUCTOR: *const c_void = core::ptr::null();

        let code = unsafe { sqlite3_bind_blob(self.raw, index, ptr, len, DESTRUCTOR) };
        match Error::new(code) {
            Error::OK => Ok(()),
            e => Err(e),
        }
    }

    /// Wrapper of C function [`sqlite3_bind_null`] .
    ///
    /// Calls method [`reset`] if necessary, and calls [`sqlite3_bind_null`] .
    /// Note that `index` starts at 1, not 0.
    ///
    /// [`reset`]: #method.reset
    /// [`step`]: #method.step
    /// [`sqlite3_bind_null`]: https://www.sqlite.org/c3ref/bind_blob.html
    /// [`sqlite3_reset`]: https://www.sqlite.org/c3ref/reset.html
    /// [`sqlite3_step`]: https://www.sqlite.org/c3ref/step.html
    #[inline]
    pub fn bind_null(&mut self, index: usize) -> Result<(), Error> {
        // self.reset() was not called after self.step() returns true.
        if self.is_row {
            self.reset();
        }

        let index = c_int::try_from(index).map_err(|_| Error::new(SQLITE_RANGE))?;
        let code = unsafe { sqlite3_bind_null(self.raw, index) };
        match Error::new(code) {
            Error::OK => Ok(()),
            e => Err(e),
        }
    }

    /// Wrapper of C function [`sqlite3_column_type`] and [`sqlite3_column_int64`] .
    ///
    /// This method calls [`sqlite3_column_type`] first.
    ///
    /// If the value type is Null, returns `None` , or if the value type is Integer, calls
    /// [`sqlite3_column_int64`] and returns the result.
    ///
    /// Note that `index` starts at 0, not 1.
    ///
    /// # Panics
    ///
    /// Panics if the previous [`step`] did not returns `true` or [`step`] did not called.
    ///
    /// Panics if `index` is out of range.
    ///
    /// Panics if the column value type is neither Null nor Integer.
    ///
    /// [`step`]: #method.step
    /// [`sqlite3_column_type`]: https://www.sqlite.org/c3ref/column_blob.html
    /// [`sqlite3_column_int64`]: https://www.sqlite.org/c3ref/column_blob.html
    #[inline]
    pub fn column_int(&mut self, index: usize) -> Option<i64> {
        assert_eq!(true, self.is_row);
        assert!(index < (self.column_count as usize));

        let index = index as c_int;
        unsafe {
            match sqlite3_column_type(self.raw, index) {
                SQLITE_NULL => None,
                SQLITE_INTEGER => Some(sqlite3_column_int64(self.raw, index)),
                _ => panic!("Bad column type"),
            }
        }
    }
}
