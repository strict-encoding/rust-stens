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

use amplify::confinement::{self, Confined, MediumOrdMap, SmallOrdSet, U32};
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
    pub fn with(lib: LibName, name: TypeName) -> TypeFqn { TypeFqn { lib, name } }
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
    /// Original type name.
    pub name: Option<TypeName>,
    /// Type origin providing information from which libraries the type is originating from. The
    /// origin information may be empty for the unnamed types. Multiple symbols are possible
    /// when several libraries define a type with the same semantic structure.
    pub orig: SmallOrdSet<LibName>,
    /// Type definition, potentially referencing other types via semantic type id.
    pub ty: Ty<SemId>,
}

impl SymTy {
    pub fn named(lib_name: LibName, ty_name: TypeName, ty: Ty<SemId>) -> SymTy {
        Self::with(Some(ty_name), Some(lib_name), ty)
    }

    pub fn unnamed(ty: Ty<SemId>) -> SymTy { Self::with(None, vec![], ty) }

    pub fn with(
        name: Option<TypeName>,
        orig: impl IntoIterator<Item = LibName>,
        ty: Ty<SemId>,
    ) -> SymTy {
        let orig = Confined::try_from_iter(orig).expect(
            "number of original libraries provided to `TypeInfo::with` must not exceed 256",
        );
        SymTy { name, orig, ty }
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
}

impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {:+}", self.id())?;
        writeln!(f)?;
        for (id, ty) in &self.0 {
            writeln!(f, "data {id:0} :: {:0}", ty)?;
        }
        Ok(())
    }
}

/*
impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {:+}", self.id())?;
        writeln!(f)?;
        for (id, info) in &self.0 {
            let ty = info.ty.clone().translate(&mut (), self).expect("type system inconsistency");
            if info.orig.is_empty() {
                writeln!(f, "data {id:0} :: {:0}", ty)?;
            } else {
                writeln!(f, "-- {id:0}")?;
                for fqn in &info.orig {
                    writeln!(f, "data {fqn} :: {:0}", ty)?;
                }
            }
        }
        Ok(())
    }
}
 */

#[cfg(feature = "base64")]
impl fmt::UpperHex for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use base64::Engine;

        let id = self.id();

        writeln!(f, "-----BEGIN STRICT TYPE SYSTEM-----")?;
        writeln!(f, "Id: {}", id)?;
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
