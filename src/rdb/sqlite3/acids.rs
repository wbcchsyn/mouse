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
use crate::data_types::Id;
use std::borrow::Borrow;

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

/// Inserts each [`Id`] of `acids` with NULL "chain_height" into RDB table "acids" if the [`Id`] is
/// not in the table yet.
/// (NULL "chain_height" represents mempool.)
///
/// [`Id`]: crate::data_types::Id
pub fn accept_to_mempool<I, S, A>(acids: I, session: &mut S) -> Result<(), Error>
where
    I: Iterator<Item = A>,
    S: Master,
    A: Borrow<Id>,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str = r#"INSERT INTO acids (id) VALUES (?1) ON CONFLICT DO NOTHING"#;
    let stmt = session.con.stmt(SQL)?;

    for id in acids {
        let id = id.borrow();
        stmt.bind_blob(1, id.as_ref())?;
        stmt.step()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_types::CryptoHash;
    use crate::rdb::sqlite3::{master, Environment};

    const ACID_COUNT: usize = 10;

    fn ids() -> Vec<Id> {
        let mut id = Id::zeroed();
        let mut ret = Vec::with_capacity(ACID_COUNT);

        for i in 1..=ACID_COUNT {
            id[0] = i as u8;
            ret.push(id);
        }

        ret
    }

    fn empty_table() -> Environment {
        let env = Environment::default();
        {
            let mut session = master(&env);
            let session = Sqlite3Session::as_sqlite3_session(&mut session);
            create_table(session).unwrap();
        }
        env
    }

    #[test]
    fn create_table_() {
        let env = Environment::default();
        let mut session = master(&env);
        let session = Sqlite3Session::as_sqlite3_session(&mut session);

        assert_eq!(true, create_table(session).is_ok());
        assert_eq!(true, create_table(session).is_ok());
    }

    #[test]
    fn accept_to_mempool_() {
        let env = empty_table();
        let mut session = master(&env);

        // Insert empty ids
        {
            let ids: &[Id] = &[];
            assert_eq!(true, accept_to_mempool(ids.iter(), &mut session).is_ok());
        }

        // Insert single id
        {
            let ids = ids();
            assert_eq!(
                true,
                accept_to_mempool(ids[0..1].iter(), &mut session).is_ok()
            );
        }

        // Insert more than 2 ids
        {
            let ids = ids();
            assert_eq!(true, accept_to_mempool(ids.iter(), &mut session).is_ok());
        }
    }
}
