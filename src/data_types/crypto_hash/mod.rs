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

//! `crypto_hash` defines traits and structs relating to cryptographic hash.

mod sha256;

use core::mem::MaybeUninit;

pub use sha256::{Sha256, Sha256Hasher};

/// Traits for wrapper of `[u8]` indicates crypto hash like 'sha256'.
pub trait CryptoHash: Sized + Clone + Copy + PartialOrd + Ord {
    /// Type of CryptoHasher to calculate this type.
    type Hasher: CryptoHasher<Hash = Self>;

    /// The number of byte count of this hash type.
    const LEN: usize;

    /// Creates a new instance filled with 0.
    #[inline]
    fn zeroed() -> Self {
        // Assume the implementation is just a wrapper of '[u8]' and don't have any other property.
        unsafe {
            let mut ret = MaybeUninit::uninit();
            let ptr = ret.as_mut_ptr() as *mut u8;
            ptr.write_bytes(0, Self::LEN);
            ret.assume_init()
        }
    }

    /// Copies `bytes` and creates a new instance.
    ///
    /// # Safety
    ///
    /// The behavior is undefined if `bytes.len` does not equal to `Self::LEN` .
    #[inline]
    unsafe fn copy_bytes(bytes: &[u8]) -> Self {
        // Assume the implementation is just a wrapper of '[u8]' and don't have any other property.
        let mut ret = MaybeUninit::uninit();

        let ptr = ret.as_mut_ptr() as *mut u8;
        ptr.copy_from_nonoverlapping(bytes.as_ptr(), Self::LEN);

        ret.assume_init()
    }

    /// Calculates crypto hash of `bytes` and returns a new instance.
    fn calculate(bytes: &[u8]) -> Self {
        <Self::Hasher as CryptoHasher>::calculate(bytes)
    }

    /// Returns the number of byte count of this hash type.
    #[inline]
    fn len(&self) -> usize {
        Self::LEN
    }

    /// Provides a raw pointer to the wrapped `[u8]` .
    #[inline]
    fn as_ptr(&self) -> *const u8 {
        // Assume the implementation is just a wrapper of '[u8]' and don't have any other property.
        let ptr = self as *const Self;
        ptr as *const u8
    }

    /// Provides a mutable raw pointer to the wrapped `[u8]` .
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        // Assume the implementation is just a wrapper of '[u8]' and don't have any other property.
        let ptr = self as *mut Self;
        ptr as *mut u8
    }
}

/// Trait for CryptoHash Calculator.
pub trait CryptoHasher: Default {
    /// Type of CryptoHash to be calculated.
    type Hash: CryptoHash;

    /// Appends `bytes` to be calculated.
    fn write(&mut self, bytes: &[u8]);

    /// Calculates the inputs and creates hash.
    fn finish(&self) -> Self::Hash;

    /// Calculates the crypto hash of `bytes` and returns the result.
    fn calculate(bytes: &[u8]) -> Self::Hash {
        let mut hasher = Self::default();
        hasher.write(bytes);
        hasher.finish()
    }
}
