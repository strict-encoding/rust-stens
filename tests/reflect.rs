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

#[macro_use]
extern crate strict_encoding;

use strict_encoding::STRICT_TYPES_LIB;
use strict_types::stl::std_stl;
use strict_types::typesys::SystemBuilder;
use strict_types::{LibBuilder, TypeLib, TypeTree};

#[test]
fn reflect() {
    let std = std_stl();
    let builder =
        LibBuilder::new(libname!(STRICT_TYPES_LIB), [std.to_dependency()]).transpile::<TypeLib>();
    let lib = builder.compile().unwrap();

    let builder = SystemBuilder::new().import(lib).unwrap().import(std).unwrap();
    match builder.finalize() {
        Ok(sys) => {
            println!("{sys}");
            println!("{sys:X}");
        }
        Err(errors) => {
            for err in errors {
                eprintln!("Error: {err}");
            }
            panic!()
        }
    }
}

#[test]
fn reflect_lib() { let _tt = TypeTree::transpile::<TypeLib>(); }
