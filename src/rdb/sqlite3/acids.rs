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

use super::{Error, Master, Slave, Sqlite3Session};
use crate::data_types::{ChainIndex, CryptoHash, Id};
use std::borrow::Borrow;
use std::collections::HashMap;

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

/// Makes each element of `acids` belong to `chain_index` if it is in mempool or does nothing, and
/// returns the number of changed acids.
///
/// # Safety
///
/// The behavior is undefined if `chain_index` is not in the "main_chain".
pub unsafe fn mempool_to_chain<I, S, A>(
    chain_index: &ChainIndex,
    acids: I,
    session: &mut S,
) -> Result<usize, Error>
where
    I: Iterator<Item = A>,
    S: Master,
    A: Borrow<Id>,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str =
        r#"UPDATE acids SET chain_height = ?1 WHERE id = ?2 AND chain_height IS NULL"#;
    let stmt = session.con.stmt(SQL)?;
    stmt.bind_int(1, chain_index.height())?;

    let mut ret = 0;

    for id in acids {
        let id = id.borrow();
        stmt.bind_blob(2, id.as_ref())?;
        stmt.step()?;

        ret += stmt.last_changes();
    }

    Ok(ret)
}

/// Moves acids included in `chain_index` to mempool, and returns the number of acids to be moved.
///
/// # Safety
///
/// The behavior is undefined if `chain_index` is not in the "main_chain".
pub unsafe fn chain_to_mempool<S>(chain_index: &ChainIndex, session: &mut S) -> Result<usize, Error>
where
    S: Master,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str = r#"UPDATE acids SET chain_height = NULL WHERE chain_height = ?1"#;
    let stmt = session.con.stmt(SQL)?;

    stmt.bind_int(1, chain_index.height())?;
    stmt.step()?;

    Ok(stmt.last_changes())
}

/// Fetches the state of each acid in `acids` .
///
/// For each [`Id`] in `acids` ,
///
/// - If the acid with the [`Id`] is in mempool, the value with the key [`Id`] is `None` .
/// - If the acid with the [`Id`] belongs to a Block in main chain, the value with the key [`Id`]
///   is [`ChainIndex`] of the Block.
/// - If the acid with the [`Id`] is neither in mempool nor in any Block in main chain, the return
///   value does not have the key [`Id`] .
///
/// [`Id`]: crate::data_types::Id
pub fn fetch_state<I, S, A>(
    acids: I,
    session: &mut S,
) -> Result<HashMap<Id, Option<ChainIndex>>, Error>
where
    I: Iterator<Item = A>,
    S: Slave,
    A: Borrow<Id>,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str = r#"SELECT acids.chain_height, main_chain.id FROM acids
    LEFT OUTER JOIN main_chain ON acids.chain_height = main_chain.height
    WHERE acids.id = ?1"#;
    let stmt = session.con.stmt(SQL)?;

    let mut ret = match acids.size_hint() {
        (n, None) => HashMap::with_capacity(n),
        (_, Some(n)) => HashMap::with_capacity(n),
    };

    for id in acids {
        let id = id.borrow();
        stmt.bind_blob(1, id.as_ref())?;
        if stmt.step()? {
            let height = stmt.column_int(0);
            match stmt.column_blob(1) {
                None => {
                    ret.insert(*id, None);
                }
                Some(id_) => {
                    let height = height.unwrap();
                    let id_ = unsafe { Id::copy_bytes(id_) };
                    ret.insert(*id, Some(ChainIndex::new(height, &id_)));
                }
            }
        }
    }

    Ok(ret)
}

/// Fetches at most `limit` number of [`Acid`] from mempool in order of the record sequence number,
/// and returns a slice of `(record sequence number, the id of the acid)` .
///
/// If `min_seq` is not `None` , this method ignores [`Acid`] whose sequence number is less than
/// `min_seq` .
///
/// [`Acid`]: crate::data_types::Acid
pub fn fetch_mempool<S>(
    min_seq: Option<i64>,
    limit: u32,
    session: &mut S,
) -> Result<impl AsRef<[(i64, Id)]>, Error>
where
    S: Slave,
{
    let session = Sqlite3Session::as_sqlite3_session(session);

    const SQL: &'static str = r#"SELECT seq, id FROM acids
    WHERE chain_height IS NULL AND seq >= ?1 ORDER BY seq ASC LIMIT ?2"#;
    let stmt = session.con.stmt(SQL)?;

    let min_seq = min_seq.unwrap_or(0);
    stmt.bind_int(1, min_seq)?;
    stmt.bind_int(2, limit as i64)?;

    let mut ret = Vec::with_capacity(limit as usize);

    while stmt.step()? {
        let seq = stmt.column_int(0).unwrap();
        let id = unsafe { Id::copy_bytes(stmt.column_blob(1).unwrap()) };
        ret.push((seq, id));
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdb::sqlite3::{main_chain, master, Environment};

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

            main_chain::create_table(session).unwrap();
            create_table(session).unwrap();
        }
        env
    }

    fn filled_table() -> Environment {
        let env = empty_table();
        accept_to_mempool(ids().iter(), &mut master(&env)).unwrap();
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

    #[test]
    fn mempool_to_chain_() {
        let env = filled_table();
        let mut session = master(&env);

        let chain_index = ChainIndex::new(1, &Id::zeroed());
        assert_eq!(Ok(1), unsafe {
            mempool_to_chain(&chain_index, ids()[0..1].iter(), &mut session)
        });

        assert_eq!(Ok(ACID_COUNT - 1), unsafe {
            mempool_to_chain(&chain_index, ids().iter(), &mut session)
        });

        assert_eq!(Ok(0), unsafe {
            mempool_to_chain(&chain_index, ids().iter(), &mut session)
        });
    }

    #[test]
    fn chain_to_mempool_() {
        let env = filled_table();
        let mut session = master(&env);
        let chain_index = ChainIndex::new(1, &Id::zeroed());

        assert_eq!(Ok(0), unsafe {
            chain_to_mempool(&chain_index, &mut session)
        });

        unsafe { mempool_to_chain(&chain_index, ids().iter(), &mut session).unwrap() };
        assert_eq!(Ok(ACID_COUNT), unsafe {
            chain_to_mempool(&chain_index, &mut session)
        });
    }

    #[test]
    fn fetch_state_from_empty_table() {
        let env = empty_table();
        let mut session = master(&env);

        let fetched = fetch_state(ids().iter(), &mut session);
        assert_eq!(true, fetched.is_ok());

        let fetched = fetched.unwrap();
        assert_eq!(true, fetched.is_empty());
    }

    #[test]
    fn fetch_state_from_filled_table() {
        let env = filled_table();
        let mut session = master(&env);

        let chain_index = ChainIndex::new(1, &Id::zeroed());
        main_chain::push(&chain_index, &mut session).unwrap();
        unsafe { mempool_to_chain(&chain_index, ids()[0..5].iter(), &mut session).unwrap() };

        let fetched = fetch_state(ids().iter(), &mut session);
        assert_eq!(true, fetched.is_ok());

        let fetched = fetched.unwrap();
        for id in &ids()[0..5] {
            assert_eq!(Some(chain_index), fetched[id]);
        }
        for id in &ids()[5..] {
            assert_eq!(None, fetched[id]);
        }
    }
}
