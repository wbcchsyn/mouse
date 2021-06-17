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

//! `acid` defines trait `Acid` and `Id` .

mod cacid;

use crate::data_types::Resource;
pub use cacid::CAcid;
use core::any::TypeId;
use std::borrow::Cow;
use std::error::Error;

#[cfg(feature = "sha256_id")]
/// `Id` is an alias to [`CryptoHash`] and used as unique id of [`Acid`] .
///
/// [`CryptoHash`]: crate::data_types::CryptoHash
pub type Id = super::crypto_hash::Sha256;

/// `Acid` is an atomic manipulation.
///
/// `Acid` corresponds to RDB transaction, however, the word 'transaction' is misreading in
/// Blockchain space, and it is named as `Acid` .
/// 'Acid' stands for 'Atomically, Consistency, Isolation, and Durability.'
///
/// In case of Bitcoin, for example, transaction and block are `Acid` .
///
/// `Acid` is constituted of 2 kinds of data; intrinsic data and extrinsic data.
/// Intrinsic data must be immutable while extrinsic data may be updated (it depends on the
/// implementation.)
///
/// Extrinsic data is a expensive calculation cache and so on. `Acid` instance must be
/// deserialized only from the intrinsic data.
/// (Usually only the intrinsic data will be shared among different nodes via P2P.)
///
/// # Id
///
/// `Id` is a crypto hash to identify `Acid` . In case of Bitcoin, for example, 'sha256 hash' is
/// used. (Actually, Bitcoin calculate sha256 twice, however, it is still a sha256 hash.)
///
/// Acid instances are regarded as same if the Id are same.
///
/// `Id` must be unique in each Blockchain and must not depends on the extrinsic data.
///
/// See also [`Id`] .
///
/// # Parent
///
/// `Acid` may depend on some other `Acid` instance(s). Parent is such a dependent `Acid` .
///
/// In case of Bitcoin, for example, blocks except for the genesis block depends on the previous
/// block and holding transactions. ('Genesis block' means the first block, which is hard corded.)
/// So block has the following parents.
///
/// - The previous block (except for the Genesis block.)
/// - Transactions that belongs to the block.
///
/// `Parent` must not depends on the extrinsic data.
///
/// # Resource
///
/// `Acid` may consume or generate some [`Resource`] .
///
/// For example, if wallet A sends some asset to wallet B, the `Acid` consumes the asset of A and
/// generates asset of B.
///
/// [`Resource`] represents consuming asset if the value is less than 0; otherwise it represents
/// generating asset.
///
/// `Resource` must not depends on the extrinsic data.
///
/// # Traceability
///
/// `Acid` is traceable if it has no parent or if all the parents are known and traceable.
/// ('Known' means the data is stored in the KVS.)
/// In other words, 'traceable' means the node knows all the ancestors if any.
///
/// Mouse node cannot make sure the `Acid` is acceptable or not if the `Acid` is not traceable.
///
/// `Traceability` may depends on the extrinsic data.
///
/// # Orphan
///
/// `Orphan` means 'not traceable'.
///
/// `Orphan` may depends on the extrinsic data.
///
/// # Validity
///
/// It is sometimes clear that the `Acid` is not acceptable and never to be recorvered. For
/// example, bad signature, some of the parents are invalid, and so on.
///
/// Such `Acid` instance should be invalidated.
///
/// `Validity` may depends on the extrinsic data.
///
/// [`Resource`]: crate::data_types::Resource
pub trait Acid {
    /// Provides `Id` of `self` .
    ///
    /// This method should be functional; it must always returns same result.
    fn id(&self) -> &Id;

    /// Serializes the immutable intrinsic data.
    ///
    /// This method should be functional; it must always returns same result.
    fn intrinsic(&self) -> Cow<[u8]>;

    /// Serializes the mutable extrinsic data.
    fn extrinsic(&self) -> Cow<[u8]>;

    /// Returns how many parents that `self` has.
    ///
    /// This method should be functional; it must always returns same result.
    fn parent_count(&self) -> usize;

    /// Returns `index` th parent if any; otherwise, i.e. `index` is greater than or equals to
    /// `self.parent_count()`, None.
    ///
    /// This method should be functional; it must always returns same result if `index` is same.
    fn parent(&self, index: usize) -> Option<Id>;

    /// Returns how many resources that `self` consumes and generates.
    ///
    /// This method should be functional: it must always returns the same result.
    fn resource_count(&self) -> usize;

    /// Returns `index` th resource that `self` consumes or generates if any, or `None` .
    ///
    /// This method should be functional; it must always returns same result if `index` is same.
    fn resource(&self, index: usize) -> Option<Resource>;

    /// Returns true if it is sure that the node knows all the ancestors; or false.
    /// i.e. this method returns true if one of the following conditions is satisfied, or false.
    ///
    /// - `self` has no parent
    /// - It is sure that all the parents are stored in the KVS, and that they (= all the parents)
    ///   are traceable.
    ///
    /// Note that this method may return false negative.
    fn is_traceable(&self) -> bool;

    /// Marks `self` as traceable and returns true if it was orphan (= not traceable); otherwise
    /// this method does nothing and returns false.
    ///
    /// After this method is called, method `is_traceable` returns true.
    ///
    /// # Warnings
    ///
    /// This method may update the extrinsic data even though it takes `&self` as the argument.
    fn set_traceable(&self) -> bool;

    /// Returns true if it is sure that `self` is not acceptable and that the status will never be
    /// recorvered.
    ///
    /// Note that `Acid` does not have a method to invalidate the instance, and it depends on the
    /// implementation.
    fn is_invalid(&self) -> bool;

    /// Returns the reason why `self` was invalidated if `is_invalid` returns true, or None.
    ///
    /// Note that `Acid` does not have a method to invalidate the instance, and it depends on the
    /// implementation.
    fn invalid_reason(&self) -> Option<&dyn Error>;

    /// Tries to update the extrinsic data of `self` to that of `other` as much as possible, and
    /// returns `true` if something is changed, or `false` .
    ///
    /// `self` and `other` must represent the same `Acid` , i.e. the `id` and the intrinsic data
    /// must be same.
    ///
    /// For example, if `self.is_traceable()` returned `false` and `other.is_traceable()` returned
    /// `true` , i.e. only `other` knew the instance has been traceable, this method makes `self`
    /// traceable.
    ///
    /// # Wargnings
    ///
    /// This method may update the extrinsic data of `self` even though the argument is `&self` .
    /// It depends on the implementation whether `other` may be updated or not.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `id` or `data` is different between `self` and `other` .
    unsafe fn merge(&self, other: &dyn Acid) -> bool;

    /// Returns the type of `Self` .
    ///
    /// This method works same to `std::any::Any::type_id` , except the required boundary
    /// conditions.
    ///
    /// This method must not depends on the extrinsic data.
    fn type_id(&self) -> TypeId;
}
