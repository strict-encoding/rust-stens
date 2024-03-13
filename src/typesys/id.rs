// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2024 UBIDECO Institute
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

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, FromBaid58, ToBaid58};
use encoding::StrictEncode;
use sha2::{Digest, Sha256};
use strict_encoding::STRICT_TYPES_LIB;

use crate::ast::SemCommit;
use crate::{CommitConsume, TypeSystem};

pub const TYPESYS_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:sys:v02";

#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct TypeSysId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl ToBaid58<32> for TypeSysId {
    const HRI: &'static str = "sts";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
}
impl FromBaid58<32> for TypeSysId {}
impl FromStr for TypeSysId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_baid58_str(s.trim_start_matches("urn:ubideco:"))
    }
}
impl Display for TypeSysId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_baid58().mnemonic())
        } else if f.sign_minus() {
            write!(f, "{:#}", self.to_baid58())
        } else if f.sign_aware_zero_pad() {
            write!(f, "urn:ubideco:{::<}", self.to_baid58())
        } else {
            write!(f, "urn:ubideco:{::<#}", self.to_baid58())
        }
    }
}

impl SemCommit for TypeSystem {
    fn sem_commit(&self, hasher: &mut impl CommitConsume) {
        hasher.commit_consume(self.count_libs().to_le_bytes());
        for lib_id in self.lib_ids() {
            lib_id.sem_commit(hasher);
        }
        hasher.commit_consume(self.count_types().to_le_bytes());
        for sem_id in self.sem_ids() {
            sem_id.sem_commit(hasher);
        }
    }
}

impl TypeSystem {
    pub fn id(&self) -> TypeSysId {
        let tag = Sha256::new_with_prefix(TYPESYS_ID_TAG).finalize();
        let mut hasher = Sha256::new();
        hasher.commit_consume(tag);
        hasher.commit_consume(tag);
        self.sem_commit(&mut hasher);
        TypeSysId::from_byte_array(hasher.finalize())
    }
}
