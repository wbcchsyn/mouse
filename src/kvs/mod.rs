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

//! 'kvs' module

mod leveldb;

pub use leveldb::{fetch, insert, update, Environment};
use std::borrow::Cow;
use std::error::Error;

/// Trait for query to the KVS to insert or to update.
///
/// It depends on the implementation whether the constructor starts the query or not.
pub trait WriteQuery {
    /// Returns `true` if the query has already finished, or `false` .
    ///
    /// This method does not block.
    fn is_finished(&self) -> bool;

    /// Starts query if not yet, and blocks till the query finished.
    /// If the query has already finished, returns immediately.
    fn wait(&mut self) -> Result<(), &dyn Error>;

    /// Returns error if `self` has finished, and if the query was failed; otherwise, returns
    /// `None`
    ///
    /// This method does not block.
    fn error(&self) -> Option<&dyn Error>;
}

/// `Row` represents data stored in the KVS.
pub struct Row<'a> {
    /// Intrinsic data
    pub intrinsic: Cow<'a, [u8]>,
    /// Extrinsic data
    pub extrinsic: Cow<'a, [u8]>,
}

/// Trait for query to the KVS to fetch.
///
/// It depends on the implementation whether the constructor starts the query or not.
pub trait ReadQuery {
    /// Returns `true` if the query has already finished, or `false` .
    ///
    /// This method does not block.
    fn is_finished(&self) -> bool;

    /// Starts query if not yet, and blocks till the query finished.
    /// If the query has already finished, returns immediately.
    ///
    /// This method returns `Row` if the data is found, or `None` if the query succeeded but no
    /// such data is stored in the KVS.
    fn wait(&mut self) -> Result<Option<Row>, &dyn Error>;

    /// Returns error if `self` has finished, and if the query was failed; otherwise, returns
    /// `None`
    ///
    /// This method does not block.
    fn error(&self) -> Option<&dyn Error>;
}
