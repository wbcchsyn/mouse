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

use super::{sqlite3, Stmt};
use std::collections::HashMap;

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
