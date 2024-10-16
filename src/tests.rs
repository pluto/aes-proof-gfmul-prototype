use ghash::{
    universal_hash::{KeyInit, UniversalHash},
    GHash,
};
use hex_literal::hex;

use super::*;

const LONE: [u8; 16] = hex!("80000000000000000000000000000000"); // 1
const LTWO: [u8; 16] = hex!("40000000000000000000000000000000"); // 2

const RONE: [u8; 16] = hex!("00000000000000000000000000000001"); // 1 << 127
const RTWO: [u8; 16] = hex!("00000000000000000000000000000002"); // 1 << 126

// todo: kill
const POLY: [u8; 16] = hex!("00000000000000000000000000000087");

// https://github.com/RustCrypto/universal-hashes/blob/master/ghash/tests/lib.rs//
const H: [u8; 16] = hex!("25629347589242761d31f826ba4b757b");
const X_1: [u8; 16] = hex!("4f4f95668c83dfb6401762bb2d01a262");
const X_2: [u8; 16] = hex!("d1a24ddd2721d006bbe45f20d3c9f362");

#[test]
fn test_reverse_byte() {
    assert_eq!(reverse_byte(0b00000001), 0b10000000);
    assert_eq!(reverse_byte(0b00000011), 0b11000000);
    assert_eq!(reverse_byte(0b10000000), 0b00000001);
    assert_eq!(reverse_byte(0b10000011), 0b11000001);
}

#[test]
fn test_parse_u8_as_bits() {
    assert_eq!(parse_u8_as_bits(0), [false, false, false, false, false, false, false, false]);
    assert_eq!(parse_u8_as_bits(1), [true, false, false, false, false, false, false, false]);
    assert_eq!(parse_u8_as_bits(2), [false, true, false, false, false, false, false, false]);
    assert_eq!(parse_u8_as_bits(3), [true, true, false, false, false, false, false, false]);
    assert_eq!(parse_u8_as_bits(4), [false, false, true, false, false, false, false, false]);
}

#[test]
fn test_parse() {
    assert_eq!(parse_array_as_pair(&RONE), (0, 1 << 63));
    assert_eq!(parse_array_as_pair(&LONE), (1, 0));
    assert_eq!(parse_array_as_pair(&RTWO), (0, 1 << 62));
    assert_eq!(parse_array_as_pair(&LTWO), (2, 0));
}

// reference rust-crypto snippet: https://github.com/RustCrypto/universal-hashes/blob/master/ghash/tests/lib.rs
fn ghash_helper(h: &[u8; 16], block: &[u8; 16]) {
    let mut ghash_rc = GHash::new(h.into());

    // 1: naive: need to specify type into casts to
    //
    // ghash_rc.update(&[*block.into()]); // type error
    //
    // note that into can infer type, calling the same method with a const value for block
    // const RONE: [u8; 16] = hex!("80000000000000000000000000000000"); // 1 << 127
    // ghash_rc.update(&[RONE.into()]); // works

    // 2: mismatch version of GenericArray with ghash
    //
    // use generic_array::{typenum::U16, GenericArray};
    // let block_array: GenericArray<u8, U16> = *GenericArray::from_slice(block); // Convert `block`
    // explicitly ghash_rc.update(&[block_array]);

    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&block]));
}

// because of generic array, it's hard to write a helper for this function
#[test]
fn test_ghash_lsb_lsb() {
    let mut ghash_rc = GHash::new(&LONE.into());
    ghash_rc.update(&[LONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LONE, &[&LONE]));
}

#[test]
fn test_ghash_msb_msb() {
    let mut ghash_rc = GHash::new(&RONE.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&RONE, &[&RONE]));
}

#[test]
fn test_ghash_two_two() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[LTWO.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&LTWO]));
}

#[test]
fn test_ghash_lsb_msb() {
    let mut ghash_rc = GHash::new(&LONE.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LONE, &[&RONE]));
}

#[test]
fn test_ghash_two_lsb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[LONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&LONE]));
}

#[test]
fn test_ghash_two_msb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&RONE]));
}

#[test]
fn test_ghash_lsb_poly() {
    let mut ghash_rc = GHash::new(&LONE.into());
    ghash_rc.update(&[POLY.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LONE, &[&POLY]));
}

#[test]
fn test_ghash_msb_poly() {
    let mut ghash_rc = GHash::new(&RONE.into());
    ghash_rc.update(&[POLY.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&RONE, &[&POLY]));
}

#[test]
fn test_ghash_ltwo_msb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&RONE]));
}

#[test]
fn test_ghash_ltwo_lsb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[LONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&LONE]));
}

// #[test]
fn test_ghash_two_block() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into(), X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&X_1, &X_2]));
}
