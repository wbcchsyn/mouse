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

//! `resource` defines struct `Resource` and relatings.

use core::mem::MaybeUninit;

/// The total buffer size of the `ResourceId` .
pub const RESOURCE_ID_BUFFER_CAPACITY: usize = 118; // The total size of 'Resource' will be 128.

/// `ResourceId` is constituted of 'owner' and 'asset type', and identifies unique [`Resource`] .
///
/// # Owner
///
/// 'Owner' is `&[u8]` .
///
/// ## 'UTXO type' Blockchain (like 'Bitcoin')
///
/// 'UTXO' stands for 'Unspent TXOut'.
///
/// 'Owner' corresponds to 'outpoint'. ('Outpoint' is an identifier of 'TxOut'. In case of Bitcoin,
/// 'outpoint' is constituted of the hash of the 'transaction' including the 'TxOut', and of the
/// index number among the 'TxOuts' in the 'transaction'.)
///
/// `Mouse` regards 'Wallet' as a set of owners and the private keys.
///
/// ## 'Account type' Blockchain (like 'Ethereum')
///
/// 'Owner' corresponds to the 'Wallet Address'.
///
/// `Mouse` regards 'Wallet' as an owner and the private key.
///
///# Asset Type
///
/// 'Asset type' is `&[u8]` and identifier unique asset. If the blockchain does not support multi
/// asset, it should be an empty slice. (otherwise it consumes the computer resource in vain.)
///
/// # Warnings
///
/// `ResourceId` uses stack buffer not to allocate heap memory for the performance.
/// The length of the 'owner' and 'asset type' must be less than or equal to
/// [`RESOURCE_ID_BUFFER_CAPACITY`] .
///
/// [`Resource`]: struct.Resource.html
/// [`RESOURCE_ID_BUFFER_CAPACITY`]: constant.RESOURCE_ID_BUFFER_CAPACITY.html
#[derive(Clone, Copy)]
pub struct ResourceId {
    buffer: [u8; RESOURCE_ID_BUFFER_CAPACITY],
    owner_len: u8,
    asset_type_len: u8,
}

impl ResourceId {
    /// Creates a new instance from `owner` and `asset_type` .
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `owner.len() + asset_type.len()` is greater than
    /// [`RESOURCE_ID_BUFFER_CAPACITY`] .
    ///
    /// [`RESOURCE_ID_BUFFER_CAPACITY`]: constant.RESOURCE_ID_BUFFER_CAPACITY.html
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::ResourceId;
    ///
    /// let owner = &[1,2,3];
    /// let asset_type = "asset name".as_ref();
    ///
    /// let _resource_id = unsafe { ResourceId::new(owner, asset_type) };
    /// ```
    #[inline]
    pub unsafe fn new(owner: &[u8], asset_type: &[u8]) -> Self {
        debug_assert!(owner.len() + asset_type.len() <= RESOURCE_ID_BUFFER_CAPACITY);

        let mut ret: MaybeUninit<Self> = MaybeUninit::uninit();

        let to_init = &mut *ret.as_mut_ptr();

        to_init.owner_len = owner.len() as u8;
        to_init.asset_type_len = asset_type.len() as u8;

        let ptr = to_init.buffer.as_mut_ptr();
        ptr.copy_from_nonoverlapping(owner.as_ptr(), owner.len());
        let ptr = ptr.add(owner.len());
        ptr.copy_from_nonoverlapping(asset_type.as_ptr(), asset_type.len());

        ret.assume_init()
    }

    /// Provides a reference to the 'owner'.
    ///
    /// # Examples
    ///
    /// ```
    /// use mouse::data_types::ResourceId;
    ///
    /// let owner = &[1,2,3];
    /// let asset_type = "asset name".as_ref();
    ///
    /// let resource_id = unsafe { ResourceId::new(owner, asset_type) };
    /// assert_eq!(owner, resource_id.owner());
    /// ```
    #[inline]
    pub fn owner(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr(), self.owner_len as usize) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn resource_id_size() {
        // The size of 'ResourceId' should be multipiles of '8'.
        assert_eq!(0, size_of::<ResourceId>() % 8);
    }
}
