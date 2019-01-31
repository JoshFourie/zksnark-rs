pub extern crate rand;

use super::field::z251::Z251;
use super::field::FieldIdentity;
use groth16::{Random, Identity, EllipticEncryptable};
use std::iter::Sum;

pub trait Encryptable {
    type Output;

    fn encrypt(self) -> Self::Output;
    fn random() -> Self;
}

pub trait EncryptProperties {
    fn detect_root(&self) -> bool;
    fn valid(&self) -> bool;
}

impl Encryptable for Z251 {
    type Output = Z251;

    fn encrypt(self) -> Self::Output {
        let mut ret = Z251::one();
        for _ in 0..self.inner {
            ret = ret * Z251 { inner: 69 };
        }

        ret
    }
    fn random() -> Self {
        Z251 {
            inner: rand::random::<u8>() % 251,
        }
    }
}

impl EncryptProperties for Z251 {
    fn detect_root(&self) -> bool {
        *self == Self::zero()
    }
    fn valid(&self) -> bool {
        true
    }
}


impl Random for Z251 {
    fn random_elem() -> Self {
        let mut r = Z251::random();
        while r == Z251::zero() {
            r = Z251::random();
        }
        r
    }
}

impl EllipticEncryptable for Z251 {
    type G1 = Self;
    type G2 = Self;
    type GT = Self;

    fn encrypt_g1(self) -> Self::G1 {
        self * 69.into()
    }
    fn encrypt_g2(self) -> Self::G2 {
        self * 69.into()
    }
    fn exp_encrypted_g1(self, g1: Self::G1) -> Self::G1 {
        self * g1
    }
    fn exp_encrypted_g2(self, g2: Self::G2) -> Self::G2 {
        self * g2
    }
    fn pairing(g1: Self::G1, g2: Self::G2) -> Self::GT {
        g1 * g2
    }
}

impl Identity for Z251 {
    fn is_identity(&self) -> bool {
        *self == Self::zero()
    }
}

impl Sum for Z251 {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Z251::from(0), |acc, x| acc + x)
    }
}