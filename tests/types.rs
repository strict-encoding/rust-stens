// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.
// Strict encoding schema library, implementing validation and parsing of strict
// encoded data against the schema.
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2022-2023 by Ubideco Project.
//
// You should have received a copy of the Apache 2.0 License along with this
// software. If not, see <https://opensource.org/licenses/Apache-2.0>.

#[macro_use]
extern crate stens;

use std::ops::Deref;

use amplify::confinement::TinyAscii;
use stens::ast::Ty;
use stens::{StenSchema, StenType, Translate};

#[repr(u8)]
pub enum Prim {
    A = 1,
    B = 2,
}

pub enum Message {
    Init(u8),
    Ping,
    Pong,
    Connect { host: TinyAscii },
}

pub struct TypeA(u8, u16);

pub struct TypeB {
    pub one: TypeA,
    pub two: TypeA,
}

pub struct Complex {
    pub a: TypeA,
    pub b: TypeB,
    pub prim: Prim,
    pub msg: Message,
}

impl StenSchema for Prim {
    const STEN_TYPE_NAME: &'static str = "Prim";

    fn sten_ty() -> Ty<StenType> {
        Ty::enumerate(variants![
            "a" => Prim::A as u8,
            "b" => Prim::B as u8,
        ])
    }
}

impl StenSchema for Message {
    const STEN_TYPE_NAME: &'static str = "Message";

    fn sten_ty() -> Ty<StenType> {
        Ty::union(fields! [
            "init" => u8::sten_type(),
            "ping" => <()>::sten_type(),
            "pong" => <()>::sten_type(),
            "connect" => StenType::new("ConnectInner", Ty::composition(fields![
                "host" => TinyAscii::sten_type(),
            ])),
        ])
    }
}

impl StenSchema for TypeA {
    const STEN_TYPE_NAME: &'static str = "TypeA";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields![
            "0" => u8::sten_type(),
            "1" => u16::sten_type(),
        ])
    }
}

impl StenSchema for TypeB {
    const STEN_TYPE_NAME: &'static str = "TypeB";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields![
            "one" => TypeA::sten_type(),
            "two" => TypeA::sten_type(),
        ])
    }
}

impl StenSchema for Complex {
    const STEN_TYPE_NAME: &'static str = "Complex";

    fn sten_ty() -> Ty<StenType> {
        Ty::composition(fields![
            "a" => TypeA::sten_type(),
            "b" => TypeB::sten_type(),
            "prim" => Prim::sten_type(),
            "msg" => Message::sten_type(),
        ])
    }
}

#[test]
fn serialize() {
    let root = Complex::sten_type();
    let id = root.id();
    let lib = root.try_into_lib().unwrap();

    println!("{}\n", id);
    println!("{}", lib);
}
