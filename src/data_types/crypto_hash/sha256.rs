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

//! `sha256` defines struct `Sha256` and `Sha256Hasher` .

use std::borrow::Borrow;

const HASH_LEN: usize = 32;

/// `Sha256` is a wrapper of `[u8; 32]` and implements [`CryptoHash`] .
///
/// [`CryptoHash`]: trait.CryptoHash.html
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Sha256([u8; HASH_LEN]);

impl AsRef<[u8]> for Sha256 {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Sha256 {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Borrow<[u8]> for Sha256 {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
