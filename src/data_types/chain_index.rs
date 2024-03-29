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

use super::{BlockHeight, Id};

/// Represents height and id of the [`Acid`] instance which constitutes a Blockchain.
///
/// The height of the genesis block (= the first block of the Blockchain) is 1, not 0.
/// (This is because some database treat '0' as a special value.)
///
/// [`Acid`]: crate::data_types::Acid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainIndex {
    height_: BlockHeight,
    id_: Id,
}

impl ChainIndex {
    /// Creates a new instance.
    ///
    /// # Panics
    ///
    /// Panics if `height` is less than or equals to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::{ChainIndex, CryptoHash, Id};
    ///
    /// let _chain_index = ChainIndex::new(35, &Id::zeroed());
    /// ```
    #[inline]
    pub fn new(height: BlockHeight, id: &Id) -> Self {
        assert_eq!(true, 0 < height);
        Self {
            height_: height,
            id_: id.clone(),
        }
    }

    /// Returns the height of `self` .
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::{ChainIndex, CryptoHash, Id};
    ///
    /// let chain_index = ChainIndex::new(35, &Id::zeroed());
    /// assert_eq!(35, chain_index.height());
    /// ```
    #[inline]
    pub fn height(&self) -> BlockHeight {
        self.height_
    }

    /// Returns the height of `self` .
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::{ChainIndex, CryptoHash, Id};
    ///
    /// let chain_index = ChainIndex::new(35, &Id::zeroed());
    /// assert_eq!(&Id::zeroed(), chain_index.id());
    /// ```
    #[inline]
    pub fn id(&self) -> &Id {
        &self.id_
    }
}
