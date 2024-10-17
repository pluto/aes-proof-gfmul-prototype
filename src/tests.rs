use std::io::Read;

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
// const POLY: [u8; 16] = hex!("00000000000000000000000000000087"); // 135 backwards
const POLY: [u8; 16] = hex!("e1000000000000000000000000000000"); // 135

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
fn test_parse_u128_as_array() {
    assert_eq!(parse_u128_as_array(0), [0; 16]);
    assert_eq!(parse_u128_as_array(1), hex!("80000000000000000000000000000000"));
    assert_eq!(parse_u128_as_array(3), hex!("c0000000000000000000000000000000"));
    assert_eq!(parse_u128_as_array(128), hex!("01000000000000000000000000000000"));
    assert_eq!(parse_u128_as_array(256), hex!("00800000000000000000000000000000"));
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
    assert_eq!(parse_array_as_pair(&RONE), (1 << 63, 0));
    assert_eq!(parse_array_as_pair(&LONE), (0, 1));
    assert_eq!(parse_array_as_pair(&RTWO), (1 << 62, 0));
    assert_eq!(parse_array_as_pair(&LTWO), (0, 2));
}

#[test]
fn test_galois_reduce() {
    assert_eq!(galois_reduce(0), 0);
    assert_eq!(galois_reduce(1), 135);
    assert_eq!(galois_reduce(2), 270);
    assert_eq!(galois_reduce(3), 135 ^ 270);
    assert_eq!(galois_reduce(4), 540);
    assert_eq!(galois_reduce(5), 135 ^ 540);
    assert_eq!(
        galois_reduce(1u128 << 120),
        2u128.pow(120) + 2u128.pow(121) + 2u128.pow(122) + 2u128.pow(127)
    );
    assert_eq!(
        galois_reduce(1u128 << 121),
        2u128.pow(121) + 2u128.pow(122) + 2u128.pow(123) + 1 + 2 + 4 + 128
    );
    assert_eq!(
        galois_reduce((1u128 << 121) + (1u128 << 120)),
        2u128.pow(120) + 2u128.pow(127) + 2u128.pow(123) + 1 + 2 + 4 + 128
    );
}

#[test]
fn test_galois_product() {
    let mut v = vec![0; 128];
    [0, 1, 2, 7].into_iter().for_each(|i| v[i] = 1);
    assert_eq!(galois_product(0).to_vec(), v);
    let mut v = vec![0; 128];
    [120, 121, 122, 127].into_iter().for_each(|i| v[i] = 1);
    assert_eq!(galois_product(120).to_vec(), v);
    let mut v = vec![0; 128];
    [0, 1, 2, 7, 121, 122, 123].into_iter().for_each(|i| v[i] = 1);
    assert_eq!(galois_product(121).to_vec(), v);

    assert_eq!(galois_product_int(0), 135);
    assert_eq!(galois_product_int(1), 270);
    assert_eq!(galois_product_int(2), 540);
    assert_eq!(galois_product_int(3), 1080);
    assert_eq!(
        galois_product_int(121),
        2u128.pow(121) + 2u128.pow(122) + 2u128.pow(123) + 1 + 2 + 4 + 128
    );
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
fn test_ghash_lsb_two() {
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
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&LTWO, &[&RONE])));
}

#[test]
fn test_ghash_ltwo_msb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&LTWO, &[&RONE])));
}

#[test]
fn test_ghash_ltwo_lsb() {
    let mut ghash_rc = GHash::new(&LTWO.into());
    ghash_rc.update(&[LONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&LTWO, &[&LONE])));
}

#[test]
fn test_ghash_rtwo_msb() {
    let mut ghash_rc = GHash::new(&RTWO.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&RTWO, &[&RONE])));
}

#[test]
fn test_ghash_h_lone() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[LONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&LONE])));
}

#[test]
fn test_ghash_h_rone() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[RONE.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&RONE])));
}

#[test]
fn test_ghash_h_rtwo() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[RTWO.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&RTWO])));
}

#[test]
fn test_ghash_h_r_8() {
    const R: [u8; 16] = hex!("00000000000000000000000000000008");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

/// ---

#[test]
fn test_ghash_h_r_3() {
    const R: [u8; 16] = hex!("00000000000000000000000000000003");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_r_4() {
    const R: [u8; 16] = hex!("00000000000000000000000000000004");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_r_5() {
    const R: [u8; 16] = hex!("00000000000000000000000000000005");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_r_9() {
    const R: [u8; 16] = hex!("00000000000000000000000000000009");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_r_c() {
    const R: [u8; 16] = hex!("0000000000000000000000000000000c");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_r_38() {
    const R: [u8; 16] = hex!("00000000000000000000000000000038");
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[R.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&R])));
}

#[test]
fn test_ghash_h_x1() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&X_1])));
}

#[test]
fn test_ghash_h_x2() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&X_2])));
}

#[test]
fn test_ghash_two_block_1() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[X_1.into(), X_2.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&X_1, &X_2])));
}

#[test]
fn test_ghash_two_block_2() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[RONE.into(), RTWO.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&RONE, &RTWO])));
}

#[test]
fn test_ghash_two_block_3() {
    let mut ghash_rc = GHash::new(&H.into());
    ghash_rc.update(&[LONE.into(), LTWO.into()]);
    let result = ghash_rc.finalize();
    assert_eq!(hex::encode(result.as_slice()), hex::encode(ghash(&H, &[&LONE, &LTWO])));
}
