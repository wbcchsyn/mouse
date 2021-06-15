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

//! This module provides functions to manipulate RDB table "main_chain" to store [`ChainIndex`] .
//!
//! Table "main_chain" has following columns.
//! (It depends on the implementation. the real schema can be different.)
//!
//! - height: integer, unique, not null
//! - id: binary string to store [`Id`], unique, not null
//!
//! [`ChainIndex`]: crate::data_types::ChainIndex
//! [`Id`]: crate::data_types::Id

use super::{sqlite3, Master, Slave};
use crate::data_types::{BlockHeight, ChainIndex, Id};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::error::Error;

/// Insert `chain_index` into RDB table "main_chain".
///
/// This function execute like the following SQL.
/// (It depends on the implementation. The real SQL may be different.)
///
/// INSERT INTO main_chain(height, id) VALUES (`chain_index.height()`, `chain_index.id()`)
///
/// # Warnings
///
/// This method does not sanitize at all except for the table constraint.
/// (i.e. The height and the id of the `chain_index` is unique in "main_chain" if this method
/// success.)
pub fn push<S>(chain_index: &ChainIndex, session: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Master,
{
    sqlite3::main_chain::push(chain_index, session)?;
    Ok(())
}

/// Delete the heighest record in the "main_chain" if "main_chain" is not empty;
/// otherwise, does nothing.
///
/// This function execute like the following SQL.
/// (It depends on the implementation. The real SQL may be different.)
///
/// DELETE FROM main_chain ORDER BY height DESC LIMIT 1
pub fn pop<S>(session: &mut S) -> Result<(), Box<dyn Error>>
where
    S: Master,
{
    sqlite3::main_chain::pop(session)?;
    Ok(())
}

/// Fetches records corresponding to `heights` from "main_chain".
///
/// This function execute like the following SQL for each `h` in `heights`.
/// (It depends on the implementation. The real SQL may be different.)
///
/// SELECT id FROM main_chain WHERE height = `h`
pub fn fetch<I, S, H>(
    heights: I,
    session: &mut S,
) -> Result<BTreeMap<BlockHeight, Id>, Box<dyn Error>>
where
    I: Iterator<Item = H>,
    H: Borrow<BlockHeight>,
    S: Slave,
{
    match sqlite3::main_chain::fetch(heights, session) {
        Ok(m) => Ok(m),
        Err(e) => Err(Box::new(e)),
    }
}

/// Fetches a record corresponding to `height` from "main_chain" and returns the id if found, or
/// `None` .
///
/// This function execute like the following SQL.
/// (It depends on the implementation. The real SQL may be different.)
///
/// SELECT id FROM main_chain WHERE height = `height`
pub fn fetch_one<S>(height: BlockHeight, session: &mut S) -> Result<Option<Id>, Box<dyn Error>>
where
    S: Slave,
{
    match sqlite3::main_chain::fetch_one(height, session) {
        Ok(id) => Ok(id),
        Err(e) => Err(Box::new(e)),
    }
}

/// Fetches at most `limit` records, whose height is greater than or equals to `min_height` order
/// by the height from RDB table "main_chain".
///
/// This function execute like the following SQL.
/// (It depends on the implementation. The real SQL may be different.)
///
/// SELECT height, id FROM main_chain WHERE height >= `min_height`
/// ORDER BY height ASC LIMIT `limit`
///
/// The result is ordered by the height.
pub fn fetch_asc<S>(
    min_height: BlockHeight,
    limit: u32,
    session: &mut S,
) -> Result<impl AsRef<[ChainIndex]>, Box<dyn Error>>
where
    S: Slave,
{
    match sqlite3::main_chain::fetch_asc(min_height, limit, session) {
        Ok(r) => Ok(r),
        Err(e) => Err(Box::new(e)),
    }
}

/// Fetches at most `limit` records, whose height is less than or equals to `max_height` order
/// by the height desc from RDB table "main_chain".
///
/// This function execute like the following SQL.
/// (It depends on the implementation. The real SQL may be different.)
///
/// SELECT height, id FROM main_chain WHERE height <= `max_height`
/// ORDER BY height DESC LIMIT `limit`
///
/// The result is ordered by the height.
pub fn fetch_desc<S>(
    max_height: BlockHeight,
    limit: u32,
    session: &mut S,
) -> Result<impl AsRef<[ChainIndex]>, Box<dyn Error>>
where
    S: Slave,
{
    match sqlite3::main_chain::fetch_desc(max_height, limit, session) {
        Ok(r) => Ok(r),
        Err(e) => Err(Box::new(e)),
    }
}
