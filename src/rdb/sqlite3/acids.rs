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

use super::{Error, Master, Sqlite3Session};

/// Make sure to create table "acids".
///
/// This method does nothing if the table is.
pub fn create_table<S>(session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    // Create table.
    {
        const SQL: &'static str = r#"CREATE TABLE IF NOT EXISTS acids(
        seq INTEGER PRIMARY KEY,
        id BLOB UNIQUE NOT NULL,
        chain_height INTEGER DEFAULT NULL
        )"#;

        let mut stmt = session.con.stmt_once(SQL)?;
        stmt.step()?;
    }

    // Create index for column chain_height.
    {
        const SQL: &'static str =
            r#"CREATE INDEX IF NOT EXISTS chain_height_ ON acids(chain_height)"#;

        let mut stmt = session.con.stmt_once(SQL)?;
        stmt.step()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdb::sqlite3::{master, Environment};

    #[test]
    fn create_table_() {
        let env = Environment::default();
        let mut session = master(&env);
        let session = Sqlite3Session::as_sqlite3_session(&mut session);

        assert_eq!(true, create_table(session).is_ok());
        assert_eq!(true, create_table(session).is_ok());
    }
}
