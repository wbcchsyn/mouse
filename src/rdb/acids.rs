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

//! This module provides functions to manipulate RDB table "acids".
//!
//! Table "acids" has following columns.
//! (It depends on the implementation. the real schema can be different.)
//!
//! - seq: integer, auto increment (or sequence)
//! - id: binary string to store [`Id`], unique, not null
//! - chain_height: integer, default null
//!
//! Note that `chain_height` stores the height of the Blockchain including the [`Acid`] .
//! If it is none, the [`Acid`] is not mined yet and in mempool.
//!
//! [`Acid`]: crate::data_types::Acid

use super::{sqlite3, Master};
use crate::data_types::Id;
use std::borrow::Borrow;
use std::error::Error;

/// Inserts each [`Id`] of `acids` with NULL "chain_height" into RDB table "acids" if the [`Id`] is
/// not in the table yet.
/// (NULL "chain_height" represents mempool.)
///
/// This function execute like the following SQL for each id in `acids` .
/// (It depends on the implementation. The real SQL may be different.)
///
/// INSERT INTO acids (id) VALUES (`id`) ON CONFLICT DO NOTHING
///
/// [`Id`]: crate::data_types::Id
pub fn accept_to_mempool<I, S, A>(acids: I, session: &mut S) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = A>,
    S: Master,
    A: Borrow<Id>,
{
    match sqlite3::acids::accept_to_mempool(acids, session) {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
