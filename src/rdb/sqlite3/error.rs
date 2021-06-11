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

use super::{SQLITE_DONE, SQLITE_OK, SQLITE_ROW};
use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_int};

/// `Error` is a wrapper of libsqlite3 error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error {
    code: c_int,
}

impl Error {
    /// Wrapper of C "SQLITE_OK".
    pub const OK: Error = Error { code: SQLITE_OK };
    /// Wrapper of C "SQLITE_ROW".
    pub const ROW: Error = Error { code: SQLITE_ROW };
    /// Wrapper of C "SQLITE_DONE".
    pub const DONE: Error = Error { code: SQLITE_DONE };

    /// Creates a new instance.
    pub const fn new(code: c_int) -> Self {
        Self { code }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let c_msg = sqlite3_errstr(self.code);
            let msg = CStr::from_ptr(c_msg);
            f.write_str(msg.to_string_lossy().as_ref())
        }
    }
}

#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_errstr(code: c_int) -> *const c_char;
}
