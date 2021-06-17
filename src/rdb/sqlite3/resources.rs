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

/// Make sure to create table "resources".
///
/// This method does nothing if the table is.
pub fn create_table<S>(session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    // Creating table
    {
        const SQL: &'static str = r#"
        CREATE TABLE IF NOT EXISTS resources(
            owner BLOB NOT NULL,
            asset_type BLOB NOT NULL,
            value INTEGER NOT NULL,
            CONSTRAINT resource_id_ PRIMARY KEY(owner, asset_type),
            CONSTRAINT value_ CHECK (value >= 0)
        )"#;

        let mut stmt = session.con.stmt_once(SQL)?;
        stmt.step()?;
    }

    // Creating trigger to cleanup
    {
        const SQL: &'static str = r#"
        CREATE TRIGGER IF NOT EXISTS cleanup_resources
            AFTER UPDATE OF value ON resources
            FOR EACH ROW
            WHEN NEW.value = 0
            BEGIN
                DELETE FROM resources WHERE owner = old.owner AND asset_type = old.asset_type;
            END
        "#;

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

        assert_eq!(true, create_table(&mut session).is_ok());
        assert_eq!(true, create_table(&mut session).is_ok());
    }
}
