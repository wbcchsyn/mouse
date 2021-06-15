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
use crate::data_types::ChainIndex;

/// Make sure to create table "main_chain".
///
/// This method does nothing if the table is.
pub fn create_table<S>(session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    const SQL: &'static str = r#"CREATE TABLE IF NOT EXISTS main_chain(
        height INTEGER PRIMARY KEY,
        id BLOB UNIQUE NOT NULL
    )"#;

    let session = Sqlite3Session::as_sqlite3_session(session);
    let mut stmt = session.con.stmt_once(SQL)?;
    stmt.step()?;

    Ok(())
}

/// Insert `chain_index` into RDB table "main_chain".
///
/// # Warnings
///
/// This method does not sanitize at all except for the table constraint.
/// (i.e. The height and the id of the `chain_index` is unique in "main_chain" if this method
/// success.)
pub fn push<S>(chain_index: &ChainIndex, session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    const SQL: &'static str = r#"INSERT INTO main_chain (height, id) VALUES (?1, ?2)"#;
    let session = Sqlite3Session::as_sqlite3_session(session);

    let stmt = session.con.stmt(SQL)?;
    stmt.bind_int(1, chain_index.height())?;
    stmt.bind_blob(2, chain_index.id().as_ref())?;
    stmt.step()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_types::{BlockHeight, CryptoHash, Id};
    use crate::rdb::sqlite3::{master, Environment};

    const CHAIN_LEN: usize = 10;
    const MAX_CHAIN_HEIGHT: BlockHeight = 10;

    fn ids() -> Vec<Id> {
        let mut ret = Vec::with_capacity(CHAIN_LEN);
        let mut id = Id::zeroed();

        for i in 1..=CHAIN_LEN {
            id[0] = i as u8;
            ret.push(id);
        }

        ret
    }

    fn main_chain() -> Vec<ChainIndex> {
        let mut ret = Vec::with_capacity(CHAIN_LEN);
        for (i, id) in ids().iter().enumerate() {
            let chain_index = ChainIndex::new((i + 1) as BlockHeight, &id);
            ret.push(chain_index);
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
    fn push_() {
        let env = empty_table();
        let mut session = master(&env);

        for c in main_chain() {
            assert_eq!(true, push(&c, &mut session).is_ok());
            assert_eq!(false, push(&c, &mut session).is_ok());
        }

        for c in main_chain() {
            assert_eq!(false, push(&c, &mut session).is_ok());
        }
    }
}
