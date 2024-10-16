#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(non_snake_case)]
#![allow(clippy::clone_on_copy)]
#![allow(unused_mut)]

#[cfg(test)] mod tests;

pub fn ghash(hashkey: &[u8; 16], blocks: &[&[u8; 16]]) -> [u8; 16] {
    let mut x = [0u8; 16];

    for block in blocks {
        for i in 0..16 {
            x[i] ^= block[i];
        }
        x = gfmul(&x, &hashkey);
    }

    x
}

/// Multiplication over the finite field $\text{GF}(2^{128})$. Elements in this field are 128-bit
/// binary vectors, and arithmetic operations are defined modulo the irreducible polynomial:
/// $x^{128} + x^7 + x^2 + x + 1$.
pub fn gfmul(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
    let (al, ar) = parse_array_as_pair(a);
    let (bl, br) = parse_array_as_pair(b);

    let rr = ar * br;
    let lr = (al * br) ^ (ar * bl);
    let ll = al * bl;

    // upper 128..256 bits and lower 128 bits
    let (upper, lower) = (ll ^ (lr >> 64), rr ^ (lr << 64));

    // parse_u128_as_array(lower)
    parse_u128_as_array(lower ^ galois_reduce(upper))
}

/// Compute x^{128} * POLY_n (mod x^{128} + 1 + x + x^2 + x^7)
///
/// where POLY_n encodes a Galois polynomial according to the GHash convention.
///
/// e.g.
/// n=1>>0: x^128 * x^0 = 1   + x   + x^2 + x^7 ; return 135
/// n=1>>1: x^128 * x^1 = x   + x^2 + x^3 + x^8 ; return 270
/// n=1>>2: x^128 * x^2 = x^2 + x^3 + x^4 + x^9 ; return 540
fn galois_reduce(n: u128) -> u128 {
    let mut m = 0u128;

    // 126: since the product x^127 * x^127 is at most 254
    // therefore there is no element of degree greater than 126=254-126
    (0..=126).for_each(|i| {
        if n & (1 << i) != 0 {
            m ^= galois_product_int(i);
        }
    });

    m
}

/// Computes galois polynomial product (x^n)(x^7 + x^2 + x + 1) encoded as u128
///
/// n=0: [1, 1, 1, 0, 0, 0, 0, 1, 0...] => 135
/// n=1: [0, 1, 1, 1, 0, 0, 0, 0, 1, 0...] => 270
fn galois_product_int(n: u8) -> u128 {
    galois_product(n).into_iter().rev().fold(0, |acc, i| (acc << 1) | (i as u128))
}

/// Computes galois polynomial product (x^n)(x^7 + x^2 + x + 1)
/// returns an array with leading LSB.
///
/// e.g.
/// n=0: [1, 1, 1, 0, 0, 0, 0, 1, 0...]
/// n=1: [0, 1, 1, 1, 0, 0, 0, 0, 1, 0...]
fn galois_product(n: u8) -> [u8; 128] {
    assert!(n < 128);
    let mut v = [0; 128];

    for j in [0, 1, 2, 7] {
        if n + j < 128 {
            v[(n + j) as usize] ^= 1;
        } else {
            for k in [0, 1, 2, 7] {
                v[(n + j + k - 128) as usize] ^= 1;
            }
        }
    }
    v
}

/// Note that these bytes are neither BE nor LE encoded.
/// Leading bit is LSB; trailing bit is MSB.
///
/// Thus:
///     1 = [ 1, 0, 0, 0, ... 0 ]
/// 2^127 = [ 0, 0, 0, ... 0, 1 ]
///
/// if byte b = [1 0 0 0 0 0 0 0]
///
/// Return: (MSB right parsed 64-bits, LSB left parsed 64 bits))
fn parse_array_as_pair(arr: &[u8; 16]) -> (u128, u128) {
    // ghash uses reversed internal byte-order
    let arr = (*arr).into_iter().map(reverse_byte).collect::<Vec<u8>>();
    let (lower, upper) = arr.split_at(8);
    let lower = (0..8).fold(0, |acc, i| acc | (lower[i] as u128) << (i * 8));
    let upper = (0..8).fold(0, |acc, i| acc | (upper[i] as u128) << (i * 8));

    (upper, lower)
}

/// interpret 128; i.e. 0x80 as [1000 0000]
fn parse_u8_as_bits(b: u8) -> Vec<bool> { (0..8).map(|i| b >> i & 1 == 1).collect() }

/// send bits in byte to reverse order; e.g. send (192=128+64) -> 3
fn reverse_byte(b: u8) -> u8 { (0..8).fold(0, |acc, i| acc | ((b >> (7 - i)) & 1) << i) }

/// parse u128 into ghash custom reversed-byte array
fn parse_u128_as_array(n: u128) -> [u8; 16] {
    let mut arr = [0; 16];
    for i in 0..16 {
        arr[i] = reverse_byte((n >> (i * 8)) as u8);
    }
    arr
}
