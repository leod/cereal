
#![cfg_attr(feature="nightly", plugin(cereal_macros))]
#![cfg_attr(feature="nightly", feature(custom_derive, plugin))]

#[cfg_attr(not(feature="nightly"), macro_use)]
extern crate cereal;

use std::fmt::Debug;
use std::io::{Cursor, Read};
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

#[cfg(feature="nightly")]
#[derive(Debug, PartialEq, CerealData)]
enum TestEnum {
    UnitVariant1,
    TupleVariant1(TestUnitStruct, TestStruct),
    UnitVariant2,
    TupleVariant2(TestTupleStruct, usize),
    StructVariant1 {
        a: usize,
        nested: Box<TestEnum>,
    }
}

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

#[test]
#[cfg(feature="nightly")]
fn test_enum() {
    test_eq(&TestEnum::UnitVariant1);
    test_eq(&TestEnum::UnitVariant2);
    test_eq(&TestEnum::TupleVariant1(TestUnitStruct, TestStruct {
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
    }));
    let test = TestEnum::TupleVariant2(TestTupleStruct(TestStruct {
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
    }, TestUnitStruct), 11);
    test_eq(&test);
    test_eq_unsized(&TestEnum::StructVariant1 { a: 11, nested: Box::new(test) })
}

fn test_eq<T>(old: &T) where T: Debug+PartialEq+CerealData {
    let mut vec: Vec<u8> = vec![0u8; mem::size_of::<T>()];
    let mut store = Cursor::new(&mut *vec);
    old.write(&mut store).unwrap();

    store.set_position(0);

    let new = T::read(&mut store).unwrap();
    assert_eq!(*old, new);
}

#[cfg_attr(not(feature="nightly"), allow(dead_code))]
fn test_eq_unsized<T>(old: &T) where T: Debug+PartialEq+CerealData {
    let vec: Vec<u8> = vec![0u8; mem::size_of::<T>()];
    let mut store = Cursor::new(vec);
    old.write(&mut store).unwrap();

    store.set_position(0);

    let new = T::read(&mut store).unwrap();
    assert_eq!(*old, new);
}
