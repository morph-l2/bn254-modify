use crate::{
    impl_add_binop_specify_output, impl_binops_additive_specify_output,
    impl_binops_multiplicative_mixed, impl_sub_binop_specify_output, impl_sum_prod,
};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use ff::{Field, FromUniformBytes, PrimeField};
use rand::RngCore;
use sp1_intrinsics::{
    bn254::{syscall_bn254_scalar_mul, syscall_bn254_scalar_mac, syscall_bn254_scalar_muladd},
    memory::memcpy32,
};
use std::convert::TryInto;
use std::io::{self, Read, Write};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[repr(transparent)]
pub struct Fr([u32; 8]);

const MODULUS: Fr = Fr([
    0xf0000001, 0x43e1f593, 0x79b97091, 0x2833e848, 
    0x8181585d, 0xb85045b6, 0xe131a029, 0x30644e72,
]);

const MODULUS_STR: &str = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";

const GENERATOR: Fr = Fr([0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

const S: u32 = 28;

const ROOT_OF_UNITY: Fr = Fr([
    0xd34f1ed9, 0x60c37c9c, 0x3215cf6d, 0xd39329c8, 
    0x98865ea9, 0x3dd31f74, 0x03ddb9f5, 0x166d18b7,
]);

const TWO_INV: Fr = Fr([
    0xa1f0fac9, 0xf8000001, 0x9419f424, 0x3cdcb848,
    0xdc2822db, 0x40c0ac2e, 0x18322739, 0x7098d014,
]);

const ROOT_OF_UNITY_INV: Fr = Fr([
    0x0ed3e50a, 0x414e6dba, 0xb22625f5, 0x9115aba7,
    0x1bbe5871, 0x80f34361, 0x04812717, 0x4daabc26,
]);

const DELTA: Fr = Fr([
    0x870e56bb, 0xe533e9a2, 0x5b5f898e, 0x5e963f25,
    0x64ec26aa, 0xd4c86e71, 0x09226b6e, 0x22c6f0ca,
]);

static ONE: Fr = Fr::one();
static NEG_ONE: Fr = Fr([
    0x0fffffff, 0x43e1f593, 0x79b97091, 0x2833e848,
    0x8181585d, 0xb85045b6, 0xe131a029, 0x30644e72,
]);

impl Fr {
    #[inline]
    pub const fn zero() -> Self {
        Fr([0, 0, 0, 0, 0, 0, 0, 0])
    }

    #[inline]
    pub const fn one() -> Self {
        Fr([1, 0, 0, 0, 0, 0, 0, 0])
    }

    pub const fn size() -> usize {
        32
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Fr> {
        let mut tmp = [0; 8];
        for i in 0..8 {
            tmp[i] = u32::from_le_bytes(bytes[i*4..(i+1)*4].try_into().unwrap());
        }
        CtOption::new(Fr(tmp), Choice::from(1))
    }

    pub fn add(&self, rhs: &Self) -> Fr {
        let mut result = *self;
        result += rhs;
        result
    }

    pub fn sub(&self, rhs: &Self) -> Fr {
        let mut result = *self;
        result -= rhs;
        result
    }

    pub fn mul(&self, rhs: &Self) -> Fr {
        let mut result = *self;
        result *= rhs;
        result
    }

    pub fn negate(&mut self) {
        unsafe {
            let mut tmp = Fr::zero();
            syscall_bn254_scalar_muladd(&mut tmp.0, &self.0, &NEG_ONE.0, &Fr::zero().0);
            *self = tmp;
        }
    }
}

impl_binops_additive_specify_output!(Fr, Fr, Fr);
impl_binops_multiplicative_mixed!(Fr, Fr, Fr);
impl_sum_prod!(Fr);

impl AddAssign<Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: Fr) {
        self.add_assign(&rhs)
    }
}

impl<'a> AddAssign<&'a Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: &'a Fr) {
        unsafe {
            syscall_bn254_scalar_mac(&mut self.0, rhs, &ONE);
        }
    }
}

impl SubAssign<Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: Fr) {
        self.sub_assign(&rhs)
    }
}

impl<'a> SubAssign<&'a Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: &'a Fr) {
        let mut tmp = *rhs;
        tmp.negate();
        unsafe {
            syscall_bn254_scalar_mac(&mut self.0, &tmp, &ONE);
        }
    }
}

impl MulAssign<Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: Fr) {
        self.mul_assign(&rhs)
    }
}

impl<'a> MulAssign<&'a Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a Fr) {
        unsafe {
            let tmp = *self;
            *self = Fr::zero();
            syscall_bn254_scalar_muladd(&mut self.0, &tmp.0, &rhs.0, &Fr::zero().0);
        }
    }
}

impl Neg for Fr {
    type Output = Fr;
    
    #[inline]
    fn neg(self) -> Fr {
        let mut ret = self;
        ret.negate();
        ret
    }
}

impl From<u64> for Fr {
    fn from(value: u64) -> Self {
        let mut ret = Fr::zero();
        ret.0[0] = value as u32;
        ret.0[1] = (value >> 32) as u32;
        ret
    }
}

impl Field for Fr {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();

    fn double(&self) -> Fr {
        self.add(self)
    }

    fn square(&self) -> Fr {
        self.mul(self)  
    }

    fn random(_rng: impl RngCore) -> Fr {
        unimplemented!()
    }

    fn invert(&self) -> CtOption<Fr> {
        unimplemented!()
    }

    fn sqrt_ratio(_num: &Self, _div: &Self) -> (Choice, Self) {
        unimplemented!()
    }
}

impl ff::PrimeField for Fr {
    type Repr = [u8; 32];

    const NUM_BITS: u32 = 254;
    const CAPACITY: u32 = 253;
    const MODULUS: &'static str = MODULUS_STR;
    const MULTIPLICATIVE_GENERATOR: Self = GENERATOR;
    const ROOT_OF_UNITY: Self = ROOT_OF_UNITY;
    const ROOT_OF_UNITY_INV: Self = ROOT_OF_UNITY_INV;
    const TWO_INV: Self = TWO_INV;
    const DELTA: Self = DELTA;
    const S: u32 = S;

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        Self::from_bytes(&repr)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut r = [0u8; 32];
        for i in 0..8 {
            r[i*4..(i+1)*4].copy_from_slice(&self.0[i].to_le_bytes());
        }
        r
    }

    fn is_odd(&self) -> Choice {
        Choice::from((self.0[0] as u8) & 1)
    }
}

impl FromUniformBytes<64> for Fr {
    fn from_uniform_bytes(_bytes: &[u8; 64]) -> Self {
        unimplemented!()
    }
}

impl ConstantTimeEq for Fr {
    fn ct_eq(&self, other: &Self) -> Choice {
        let mut result = Choice::from(1u8);
        for i in 0..8 {
            result &= self.0[i].ct_eq(&other.0[i]);
        }
        result
    }
}

impl ConditionallySelectable for Fr {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = [0u32; 8];
        for i in 0..8 {
            result[i] = u32::conditional_select(&a.0[i], &b.0[i], choice);
        }
        Fr(result)
    }
}