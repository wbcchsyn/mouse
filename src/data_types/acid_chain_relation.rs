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

use super::{ChainIndex, Id};

/// `AcidChainRelation` represents a record in RDB table "acids".
///
/// This struct is constituted of [`Id`] of the [`Acid`], and the [`ChainIndex`] including the
/// [`Acid`].
///
/// [`Id`]: type.Id.html
/// [`Acid`]: trait.Acid.html
/// [`ChainIndex`]: struct.ChainIndex.html
pub struct AcidChainRelation {
    id_: Id,
    chain_index_: Option<ChainIndex>,
}

impl AcidChainRelation {
    /// Creates a new instance.
    ///
    /// # Arguments
    ///
    /// ## `id`
    ///
    /// The id of the [`Acid`] .
    ///
    /// ## `chain_index`
    ///
    /// [`ChainIndex`] including the [`Acid`] , or `None` if not belongs to any block in the
    /// "main_chain" .
    ///
    /// [`Id`]: type.Id.html
    /// [`Acid`]: trait.Acid.html
    /// [`ChainIndex`]: struct.ChainIndex.html
    pub const fn new(id: &Id, chain_index: Option<&ChainIndex>) -> Self {
        Self {
            id_: *id,
            chain_index_: match chain_index {
                None => None,
                Some(&c) => Some(c),
            },
        }
    }

    /// Provides a reference to the [`Id`] of the [`Acid`] .
    ///
    /// [`Id`]: type.Id.html
    /// [`Acid`]: trait.Acid.html
    pub const fn id(&self) -> &Id {
        &self.id_
    }

    /// Provides a reference to the [`ChainIndex`] including the [`Acid`] , or `None` if the
    /// [`Acid`] does not belong to any Block in "main_chain" yet.
    /// [`Acid`]: trait.Acid.html
    /// [`ChainIndex`]: struct.ChainIndex.html
    pub const fn chain_index(&self) -> Option<&ChainIndex> {
        (&self.chain_index_).as_ref()
    }
}
