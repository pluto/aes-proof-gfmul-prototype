use ghash::{
    universal_hash::{KeyInit, UniversalHash},
    GHash,
};
use hex_literal::hex;

use super::*;

const MSB: [u8; 16] = hex!("80000000000000000000000000000000"); // 1 << 127
const LTWO: [u8; 16] = hex!("40000000000000000000000000000000"); // 2
const LSB: [u8; 16] = hex!("00000000000000000000000000000001"); // 1
const TWO: [u8; 16] = hex!("00000000000000000000000000000002"); // 1 << 126
const POLY: [u8; 16] = hex!("00000000000000000000000000000087"); // todo: kill

// https://github.com/RustCrypto/universal-hashes/blob/master/ghash/tests/lib.rs//
const H: [u8; 16] = hex!("25629347589242761d31f826ba4b757b");
const X_1: [u8; 16] = hex!("4f4f95668c83dfb6401762bb2d01a262");
const X_2: [u8; 16] = hex!("d1a24ddd2721d006bbe45f20d3c9f362");

#[test]
fn test_parse() {
    let parsed = parse_input(&MSB);
    assert_eq!(parsed, (1 << 63, 0));
    let parsed = parse_input(&LSB);
    assert_eq!(parsed, (0, 1));
}

#[test]
fn sanity_checks() {
    assert_eq!(gfmul(&LSB, &LSB), LSB);
    assert_eq!(gfmul(&LSB, &TWO), TWO);
    assert_eq!(gfmul(&MSB, &LSB), MSB);
}

// reference rust-crypto snippet: https://github.com/RustCrypto/universal-hashes/blob/master/ghash/tests/lib.rs
fn ghash_helper(h: &[u8; 16], block: &[u8; 16]) {
    let mut ghash_rc = GHash::new(h.into());

    // 1: naive: need to specify type into casts to
    //
    // ghash_rc.update(&[*block.into()]); // type error
    //
    // note that into can infer type, calling the same method with a const value for block
    // const MSB: [u8; 16] = hex!("80000000000000000000000000000000"); // 1 << 127
    // ghash_rc.update(&[MSB.into()]); // works

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
fn overflow_check() {
    assert_eq!(gfmul(&MSB, &TWO), POLY);
}

#[test]
fn test_ghash_lsb_lsb() {
    let mut ghash_rc = GHash::new(&LSB.into());
    ghash_rc.update(&[LSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LSB, &[&LSB]));
}

#[test]
fn test_ghash_msb_msb() {
    let mut ghash_rc = GHash::new(&MSB.into());
    ghash_rc.update(&[MSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&MSB, &[&MSB]));
}

#[test]
fn test_ghash_two_two() {
    let mut ghash_rc = GHash::new(&TWO.into());
    ghash_rc.update(&[TWO.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&TWO, &[&TWO]));
}

#[test]
fn test_ghash_lsb_msb() {
    let mut ghash_rc = GHash::new(&LSB.into());
    ghash_rc.update(&[MSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LSB, &[&MSB]));
}

#[test]
fn test_ghash_two_lsb() {
    let mut ghash_rc = GHash::new(&TWO.into());
    ghash_rc.update(&[LSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&TWO, &[&LSB]));
}

#[test]
fn test_ghash_two_msb() {
    let mut ghash_rc = GHash::new(&TWO.into());
    ghash_rc.update(&[MSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&TWO, &[&MSB]));
}

#[test]
fn test_ghash_lsb_poly() {
    let mut ghash_rc = GHash::new(&LSB.into());
    ghash_rc.update(&[POLY.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LSB, &[&POLY]));
}

#[test]
fn test_ghash_msb_poly() {
    let mut ghash_rc = GHash::new(&MSB.into());
    ghash_rc.update(&[POLY.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&MSB, &[&POLY]));
}

#[test]
fn test_ghash_ltwo_msb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[MSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&MSB]));
}

#[test]
fn test_ghash_ltwo_lsb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[LSB.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&LTWO, &[&LSB]));
}

// #[test]
fn test_ghash_two_block() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into(), X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&X_1, &X_2]));
}
