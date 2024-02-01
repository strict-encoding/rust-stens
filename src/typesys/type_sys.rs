// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::fmt::{self, Display, Formatter};
use std::ops::Index;

use amplify::confinement::{self, MediumOrdMap};
use amplify::num::u24;
use encoding::{LibName, StrictDeserialize, StrictSerialize, TypeName};
use strict_encoding::STRICT_TYPES_LIB;

use crate::{SemId, Ty};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib}.{name}")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeFqn {
    pub lib: LibName,
    pub name: TypeName,
}

impl TypeFqn {
    pub fn with(lib: impl Into<LibName>, name: impl Into<TypeName>) -> TypeFqn {
        TypeFqn {
            lib: lib.into(),
            name: name.into(),
        }
    }
}

impl From<&'static str> for TypeFqn {
    fn from(value: &'static str) -> Self {
        let Some((lib, name)) = value.split_once('.') else {
            panic!("invalid fully qualified type name `{value}`");
        };
        TypeFqn {
            lib: LibName::from(lib),
            name: TypeName::from(name),
        }
    }
}

/// Type coupled with symbolic information.
#[derive(Clone, Eq, PartialEq, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct SymTy {
    /// Type origin providing information from which library and under which name the type is
    /// originating from. The origin information may be empty for the unnamed types.
    pub orig: Option<TypeFqn>,
    /// Type definition, potentially referencing other types via semantic type id.
    pub ty: Ty<SemId>,
}

impl SymTy {
    pub fn named(lib_name: LibName, ty_name: TypeName, ty: Ty<SemId>) -> SymTy {
        Self::with(Some(TypeFqn::with(lib_name, ty_name)), ty)
    }

    pub fn unnamed(ty: Ty<SemId>) -> SymTy { Self::with(None::<TypeFqn>, ty) }

    pub fn with(orig: Option<impl Into<TypeFqn>>, ty: Ty<SemId>) -> SymTy {
        SymTy {
            orig: orig.map(|o| o.into()),
            ty,
        }
    }
}

/// Type system represents a set of strict types assembled from multiple
/// libraries. It is designed to provide all necessary type information to
/// analyze a type with all types it depends onto.
///
/// # Type guarantees
///
/// - Total number of types do not exceed 2^24-1;
/// - Strict-serialized size is less than 2^24 bytes;
/// - A type with the same semantic id can't appear in more than 256 libraries;
/// - Type system is complete (i.e. no type references a type which is not a part of the system).
#[derive(Wrapper, Clone, Eq, PartialEq, Debug, Default, From)]
#[wrapper(Deref)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct TypeSystem(MediumOrdMap<SemId, Ty<SemId>>);

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn new() -> Self { Self::default() }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }

    pub(super) fn insert_unchecked(
        &mut self,
        sem_id: SemId,
        ty: Ty<SemId>,
    ) -> Result<bool, confinement::Error> {
        self.0.insert(sem_id, ty).map(|r| r.is_some())
    }

    pub fn get(&self, sem_id: SemId) -> Option<&Ty<SemId>> { self.0.get(&sem_id) }
}

impl Index<SemId> for TypeSystem {
    type Output = Ty<SemId>;

    fn index(&self, index: SemId) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!("type with semantic id {index} is not a part of the type system")
        })
    }
}

impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {}", self.id())?;
        writeln!(f)?;
        for (id, ty) in &self.0 {
            writeln!(f, "data {id:-} :: {:-}", ty)?;
        }
        Ok(())
    }
}

#[cfg(feature = "base64")]
impl fmt::UpperHex for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use amplify::confinement::U32;
        use base64::Engine;

        let id = self.id();

        writeln!(f, "-----BEGIN STRICT TYPE SYSTEM-----")?;
        writeln!(f, "Id: {:-}", id)?;
        writeln!(f)?;

        let data = self.to_strict_serialized::<U32>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 64 {
            let (line, rest) = data.split_at(64);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n-----END STRICT TYPE SYSTEM-----")?;
        Ok(())
    }
}
