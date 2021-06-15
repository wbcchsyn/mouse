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
