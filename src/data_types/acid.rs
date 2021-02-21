// Copyright 2020 Shin Yoshida
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

#[cfg(feature = "sha256_id")]
/// `Id` is an alias to [`CryptoHash`] and used as unique id of [`Acid`] .
///
/// [`CryptoHash`]: crypto_hash/trait.CryptoHash.html
/// [`Acid`]: trait.Acid.html
pub type Id = super::crypto_hash::Sha256;
