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
    // let hashkey = reverse_bits(hashkey);
    // let blocks = blocks.into_iter().map(|b| reverse_bits(b)).collect::<Vec<_>>();

    for block in blocks {
        for i in 0..16 {
            x[i] ^= block[i];
        }
        x = gfmul(&x, &hashkey);
    }

    x
}

// pub fn u8_reverse_bits(n: u8) -> u8 {
// aa
// aaaaa
// }

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

    // todo: map upper into lower 128 bits by galois field arithm
    (lower ^ remap(upper)).to_be_bytes()
}

/// Each bit of the argument $n$ is an overflow bit of the Galois field polynomial.
/// Compute x^{128} * n (mod x^{128} + x^7 + x^2 + x + 1)
fn remap(n: u128) -> u128 {
    let mut v = vec![0; 128];
    (0..=126).for_each(|i| {
        if n & (1 << i) == 1 {
            let m = generate_galois_field_mapping(i);
            v.iter_mut().zip(m.into_iter()).for_each(|(vj, mj)| *vj ^= mj);
        }
    });

    // BE accumululate v
    v.into_iter().fold(0, |acc, i| acc << 1 | i as u128)
}

// assume i < 128
pub fn generate_galois_field_mapping(i: u8) -> Vec<u8> {
    assert!(i < 128);
    let mut v: Vec<u8> = vec![0; 128];
    for j in [0, 1, 2, 7] {
        if i + j < 128 {
            v[(i + j) as usize] ^= 1;
        } else {
            for k in [0, 1, 2, 7] {
                v[(i + j + k - 128) as usize] ^= 1;
            }
        }
    }
    v.reverse();
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
/// Return: (left parsed 64-bits, right parsed 64 bits)
pub fn parse_array_as_pair(arr: &[u8; 16]) -> (u128, u128) {
    let arr = (*arr).into_iter().map(reverse_byte).collect::<Vec<u8>>();
    let (l, r) = arr.split_at(8);
    let l = (0..8).fold(0, |acc, i| acc | (l[i] as u128) << (i * 8));
    let r = (0..8).fold(0, |acc, i| acc | (r[i] as u128) << (i * 8));

    (l, r)
}

/// interpret 128; i.e. 0x80 as [1000 0000]
fn parse_u8_as_bits(b: u8) -> Vec<bool> { (0..8).map(|i| b >> i & 1 == 1).collect() }

// /// send bits in byte to reverse order; e.g. send (192=128+64) -> 3
// fn reverse_byte(b: u8) -> u8 { (0..8).fold(0, |acc, i| acc | (b & (128 >> i))) }
fn reverse_byte(b: u8) -> u8 { (0..8).fold(0, |acc, i| acc | ((b >> (7 - i)) & 1) << i) }
