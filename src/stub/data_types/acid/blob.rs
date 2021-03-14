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

use crate::data_types::{Acid, CVec, Id, Resource};
use core::any::TypeId;
use std::borrow::Cow;
use std::error::Error;

/// Format
///
/// Intrinsic ::= [APPLICATION 1] OCTET STRING
struct Intrinsic {
    data: CVec<u8>,
}

/// `Blob` implements `Acid` , and represents binary data without no resource, no parents.
///
/// This must not be orphan nor invalidate.
pub struct Blob {
    id_: Id,
    intrinsic_: Intrinsic,
}

impl Acid for Blob {
    fn id(&self) -> &Id {
        &self.id_
    }

    fn intrinsic(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.intrinsic_.data.as_ref())
    }

    fn extrinsic(&self) -> Cow<[u8]> {
        Cow::default()
    }

    fn parent_count(&self) -> usize {
        0
    }

    fn parent(&self, _: usize) -> Option<Id> {
        None
    }

    fn resource_count(&self) -> usize {
        0
    }

    fn resource(&self, _: usize) -> Option<Resource> {
        None
    }

    fn is_traceable(&self) -> bool {
        true
    }

    fn set_traceable(&self) -> bool {
        false
    }

    fn is_invalid(&self) -> bool {
        false
    }

    fn invalid_reason(&self) -> Option<&dyn Error> {
        None
    }

    unsafe fn merge(&self, _other: &dyn Acid) -> bool {
        false
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}
