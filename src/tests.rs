use ghash::{
    universal_hash::{KeyInit, UniversalHash},
    GHash,
};
use hex_literal::hex;

use super::*;

const MSB: [u8; 16] = hex!("80000000000000000000000000000000");
const LSB: [u8; 16] = hex!("00000000000000000000000000000001");
const TWO: [u8; 16] = hex!("00000000000000000000000000000002");
const POLY: [u8; 16] = hex!("00000000000000000000000000000087");

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

#[test]
fn overflow_check() {
    assert_eq!(gfmul(&MSB, &TWO), POLY);
}

#[test]
fn test_ghash_one_block() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&X_1]));

    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&X_2]));
}

// #[test]
fn test_ghash_two_block() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into(), X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(result.as_slice(), ghash(&H, &[&X_1, &X_2]));
}
