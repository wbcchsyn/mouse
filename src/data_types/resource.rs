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
