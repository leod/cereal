
#![cfg_attr(feature="nightly", plugin(cereal_macros))]
#![cfg_attr(feature="nightly", feature(custom_derive, plugin))]

#[cfg_attr(not(feature="nightly"), macro_use)]
extern crate cereal;

use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::iter;
use std::mem;
use cereal::CerealData;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature="nightly", derive(CerealData))]
struct TestStruct {
    a: u8,
    b: i8,
    c: u16,
    d: i16,
    e: u32,
    f: i32,
    g: u64,
    h: i64,
    i: usize,
    j: isize,
}

#[cfg(not(feature="nightly"))]
impl_cereal_data!(TestStruct, a, b, c, d, e, f, g, h, i, j);

#[derive(Debug, PartialEq)]
#[cfg_attr(feature="nightly", derive(CerealData))]
struct TestTupleStruct(TestStruct, TestUnitStruct);

#[cfg(not(feature="nightly"))]
impl_cereal_data!(TestTupleStruct(), a, b);

#[derive(Debug, PartialEq)]
#[cfg_attr(feature="nightly", derive(CerealData))]
struct TestUnitStruct;

#[cfg(not(feature="nightly"))]
impl_cereal_data!(TestUnitStruct);

#[test]
fn test_primitives() {
    let test = TestStruct {
        a: 1,
        b: -2,
        c: 3,
        d: -4,
        e: 5,
        f: -6,
        g: 7,
        h: -8,
        i: 9,
        j: -10,
    };
    test_eq(&test);
}

#[test]
fn test_tuple_struct() {
    let test = TestTupleStruct(
        TestStruct {
            a: 1,
            b: -2,
            c: 3,
            d: -4,
            e: 5,
            f: -6,
            g: 7,
            h: -8,
            i: 9,
            j: -10,
        },
        TestUnitStruct,
    );
    test_eq(&test);
}


#[test]
fn test_unit_struct() {
    test_eq(&TestUnitStruct);
}

fn test_eq<T>(old: &T) where T: Debug+PartialEq+CerealData {
    let mut vec: Vec<u8> = iter::repeat(0u8).take(mem::size_of::<T>()).collect();
    let mut store = Cursor::new(&mut *vec);
    old.write(&mut store).unwrap();

    store.set_position(0);

    let new = T::read(&mut store).unwrap();
    assert_eq!(*old, new);
}
