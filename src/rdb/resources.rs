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

//! This module provides functions to manipulate RDB table "resources".
//!
//! Table "resources" stores the current balance of each [`ResourceId`] at main chain state.
//! (It does not take the mempool into account.)
//!
//! Table "acids" has following columns.
//! (It depends on the implementation. the real schema can be different.)
//!
//! - owner: binary string to store the owner of [`ResourceId`] .
//! - asset_type: binary string to store the asset type of [`ResourceId`] .
//! - value: The number of the asset to be depositted.
//!
//! [`ResourceId`]: crate::data_types::ResourceId

use super::{sqlite3, Master, Slave};
use crate::data_types::{AssetValue, ResourceId};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;

/// Upadtes the asset value in RDB table "resources".
///
/// `balances` is an iterator of ([`ResourceId`] , [`AssetValue`] ) or a reference to it.
///
/// For each balance in `balances` , the value of the [`ResourceId`] is increased by the
/// [`AssetValue`]; i.e. if the [`AssetValue`] is greater than 0, the value is increased
/// (depositted), or if the [`AssetValue`] is less than 0, the value is decreased (withdrawn.)
///
/// # Error
///
/// Errors if any [`AssetValue`] is less than 0.
///
/// [`ResourceId`]: crate::data_types::ResourceId
/// [`AssetValue`]: crate::data_types::AssetValue
pub fn update_balance<I, S, B, R, V>(balances: I, session: &mut S) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = B> + Clone,
    S: Master,
    B: Borrow<(R, V)>,
    R: Borrow<ResourceId>,
    V: Borrow<AssetValue>,
{
    match sqlite3::resources::update_balance(balances, session) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

/// Fetches the depositted value of each [`ResourceId`] in `resource_ids` .
///
/// The returned value does not has the [`ResourceId`] as the key if the corresponding value is 0.
pub fn fetch<I, S, R>(
    resource_ids: I,
    session: &mut S,
) -> Result<HashMap<ResourceId, AssetValue>, Box<dyn Error>>
where
    I: Iterator<Item = R>,
    S: Slave,
    R: Borrow<ResourceId>,
{
    match sqlite3::resources::fetch(resource_ids, session) {
        Ok(m) => Ok(m),
        Err(e) => Err(Box::new(e)),
    }
}
