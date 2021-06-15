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

use super::{sqlite3, Master};
use crate::data_types::ChainIndex;
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
