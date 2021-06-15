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
use crate::data_types::{BlockHeight, ChainIndex, CryptoHash, Id};
use std::borrow::Borrow;
use std::collections::BTreeMap;

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

/// Delete the heighest record in the "main_chain" if "main_chain" is not empty;
/// otherwise, does nothing.
pub fn pop<S>(session: &mut S) -> Result<(), Error>
where
    S: Master,
{
    const SQL: &'static str = r#"DELETE FROM main_chain ORDER BY height DESC LIMIT 1"#;
    let session = Sqlite3Session::as_sqlite3_session(session);

    let stmt = session.con.stmt(SQL)?;
    stmt.step()?;
    Ok(())
}

/// Fetches records corresponding to `heights` from "main_chain".
pub fn fetch<I, S, H>(heights: I, session: &mut S) -> Result<BTreeMap<BlockHeight, Id>, Error>
where
    I: Iterator<Item = H>,
    H: Borrow<BlockHeight>,
    S: Slave,
{
    const SQL: &'static str = r#"SELECT id FROM main_chain WHERE height = ?1"#;
    let session = Sqlite3Session::as_sqlite3_session(session);
    let stmt = session.con.stmt(SQL)?;

    let mut ret = BTreeMap::new();
    for h in heights {
        let h = *h.borrow();
        stmt.bind_int(1, h)?;
        if stmt.step()? {
            let id = unsafe { Id::copy_bytes(stmt.column_blob(0).unwrap()) };
            ret.insert(h, id);
        }
    }

    Ok(ret)
}

/// Fetches at most `limit` records, whose height is greater than or equals to `min_height` order
/// by the height from RDB table "main_chain".
///
/// The result is ordered by the height.
pub fn fetch_asc<S>(
    min_height: BlockHeight,
    limit: u32,
    session: &mut S,
) -> Result<impl AsRef<[ChainIndex]>, Error>
where
    S: Slave,
{
    const SQL: &'static str =
        r#"SELECT height, id FROM main_chain WHERE height >= ?1 ORDER BY height ASC LIMIT ?2"#;
    let session = Sqlite3Session::as_sqlite3_session(session);

    let stmt = session.con.stmt(SQL)?;
    stmt.bind_int(1, min_height)?;
    stmt.bind_int(2, limit as i64)?;

    let mut ret = Vec::with_capacity(limit as usize);
    while stmt.step()? {
        let height = stmt.column_int(0).unwrap();
        let id = unsafe { Id::copy_bytes(stmt.column_blob(1).unwrap()) };
        ret.push(ChainIndex::new(height, &id));
    }
    Ok(ret)
}

/// Fetches at most `limit` records, whose height is less than or equals to `max_height` order
/// by the height desc from RDB table "main_chain".
///
/// The result is ordered by the height desc.
pub fn fetch_desc<S>(
    max_height: BlockHeight,
    limit: u32,
    session: &mut S,
) -> Result<impl AsRef<[ChainIndex]>, Error>
where
    S: Slave,
{
    const SQL: &'static str =
        r#"SELECT height, id FROM main_chain WHERE height <= ?1 ORDER BY height DESC LIMIT ?2"#;
    let session = Sqlite3Session::as_sqlite3_session(session);

    let stmt = session.con.stmt(SQL)?;
    stmt.bind_int(1, max_height)?;
    stmt.bind_int(2, limit as i64)?;

    let mut ret = Vec::with_capacity(limit as usize);
    while stmt.step()? {
        let height = stmt.column_int(0).unwrap();
        let id = unsafe { Id::copy_bytes(stmt.column_blob(1).unwrap()) };
        ret.push(ChainIndex::new(height, &id));
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdb::sqlite3::{master, slave, Environment};

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

    fn filled_table() -> Environment {
        let env = empty_table();
        {
            let mut session = master(&env);

            for c in main_chain() {
                let _ = push(&c, &mut session);
            }
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
            let heights: &[BlockHeight] = &[c.height()];
            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(0, fetched.unwrap().len());

            assert_eq!(true, push(&c, &mut session).is_ok());
            assert_eq!(false, push(&c, &mut session).is_ok());

            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(1, fetched.unwrap().len());
        }

        for c in main_chain() {
            assert_eq!(false, push(&c, &mut session).is_ok());
        }
    }

    #[test]
    fn pop_() {
        let env = filled_table();
        let mut session = master(&env);

        for i in 0..MAX_CHAIN_HEIGHT {
            let heights: &[BlockHeight] = &[MAX_CHAIN_HEIGHT - i];
            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(1, fetched.unwrap().len());

            assert_eq!(true, pop(&mut session).is_ok());

            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(0, fetched.unwrap().len());
        }

        assert_eq!(true, pop(&mut session).is_ok());
    }

    #[test]
    fn fetch_from_empty() {
        let env = empty_table();
        let mut session = slave(&env);

        // Empty height
        {
            let heights: &[BlockHeight] = &[];

            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(true, fetched.is_ok());

            let fetched = fetched.unwrap();
            assert_eq!(0, fetched.len());
        }

        // Single height
        {
            for i in [-1, 0, 1].iter() {
                let heights: &[BlockHeight] = &[*i];

                let fetched = fetch(heights.iter(), &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();
                assert_eq!(0, fetched.len());
            }
        }

        // 2 heights
        {
            for i in [-1, 0, 1].iter() {
                for j in [-1, 0, 1].iter() {
                    let heights: &[BlockHeight] = &[*i, *j];

                    let fetched = fetch(heights.iter(), &mut session);
                    assert_eq!(true, fetched.is_ok());

                    let fetched = fetched.unwrap();
                    assert_eq!(0, fetched.len());
                }
            }
        }
    }

    #[test]
    fn fetch_from_filled() {
        let env = filled_table();
        let mut session = slave(&env);

        // Empty height
        {
            let heights: &[BlockHeight] = &[];

            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(true, fetched.is_ok());

            let fetched = fetched.unwrap();
            assert_eq!(0, fetched.len());
        }

        // 1 height
        for i in -1..=MAX_CHAIN_HEIGHT + 1 {
            let heights: &[BlockHeight] = &[i];

            let fetched = fetch(heights.iter(), &mut session);
            assert_eq!(true, fetched.is_ok());

            let fetched = fetched.unwrap();

            if 0 < i && i <= MAX_CHAIN_HEIGHT {
                // 1 hit
                assert_eq!(1, fetched.len());
                let expected = ids()[(i - 1) as usize];
                assert_eq!(expected, fetched[&i]);
            } else {
                // 0 hit
                assert_eq!(0, fetched.len());
            }
        }

        // 2 heights
        for i in -1..=MAX_CHAIN_HEIGHT + 1 {
            for j in -1..=MAX_CHAIN_HEIGHT + 1 {
                let heights: &[BlockHeight] = &[i, j];

                let fetched = fetch(heights.iter(), &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();

                if 0 < i && i <= MAX_CHAIN_HEIGHT {
                    // hit i
                    let expected = ids()[(i - 1) as usize];
                    assert_eq!(expected, fetched[&i]);
                } else {
                    // fault i
                    assert_eq!(false, fetched.contains_key(&i));
                }

                if 0 < j && j <= MAX_CHAIN_HEIGHT {
                    // hit j
                    let expected = ids()[(j - 1) as usize];
                    assert_eq!(expected, fetched[&j]);
                } else {
                    // fault j
                    assert_eq!(false, fetched.contains_key(&j));
                }
            }
        }
    }

    #[test]
    fn fetch_asc_from_empty_table() {
        let env = empty_table();
        let mut session = slave(&env);

        for min_height in &[-1, 0, 1] {
            for limit in &[0, 1] {
                let fetched = fetch_asc(*min_height, *limit, &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();
                assert_eq!(0, fetched.as_ref().len());
            }
        }
    }

    #[test]
    fn fetch_asc_from_filled_table() {
        let env = filled_table();
        let mut session = slave(&env);

        for min_height in -1..=(MAX_CHAIN_HEIGHT + 1) {
            for limit in 0..=(CHAIN_LEN + 1) {
                let fetched = fetch_asc(min_height, limit as u32, &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();

                let start = std::cmp::max(0, min_height - 1) as usize;
                let end = std::cmp::min(MAX_CHAIN_HEIGHT as usize, start + limit);
                let chain = main_chain();
                let expected = &chain[start..end];

                assert_eq!(expected, fetched.as_ref());
            }
        }
    }

    #[test]
    fn fetch_desc_from_empty_table() {
        let env = empty_table();
        let mut session = slave(&env);

        for max_height in &[-1, 0, 1] {
            for limit in &[0, 1] {
                let fetched = fetch_desc(*max_height, *limit, &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();
                assert_eq!(0, fetched.as_ref().len());
            }
        }
    }

    #[test]
    fn fetch_desc_from_filled_table() {
        let env = filled_table();
        let mut session = slave(&env);

        for max_height in -1..=(MAX_CHAIN_HEIGHT + 1) {
            for limit in 0..=(CHAIN_LEN + 1) {
                let fetched = fetch_desc(max_height, limit as u32, &mut session);
                assert_eq!(true, fetched.is_ok());

                let fetched = fetched.unwrap();

                let end = std::cmp::min(MAX_CHAIN_HEIGHT, max_height);
                let start = std::cmp::max(0, end - (limit as i64)) as usize;
                let end = std::cmp::max(0, end) as usize;
                let mut chain = main_chain();
                let expected = &mut chain[start..end];
                expected.reverse();

                assert_eq!(expected, fetched.as_ref());
            }
        }
    }
}
