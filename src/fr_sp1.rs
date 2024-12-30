use super::ff::*;
use core::fmt::{self, Debug, Display};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
use sp1_intrinsics;

#[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
use super::arithmetic;

const MODULUS: [u64; 4] = [
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
];

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Fr(pub [u64; 4]);

impl Fr {
    #[inline]
    pub const fn zero() -> Self {
        Fr([0, 0, 0, 0])
    }

    #[inline]
    pub const fn one() -> Self {
        Fr([1, 0, 0, 0])
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Fr> {
        let mut limbs = [0u64; 4];
        
        for i in 0..4 {
            let mut val = 0u64;
            for j in 0..8 {
                val |= (bytes[i*8 + j] as u64) << (j * 8);
            }
            limbs[i] = val;
        }

        // Check if value is less than modulus
        let mut is_less = false;
        for i in (0..4).rev() {
            if limbs[i] < MODULUS[i] {
                is_less = true;
                break;
            }
            if limbs[i] > MODULUS[i] {
                break;
            }
        }

        CtOption::new(Fr(limbs), Choice::from(is_less as u8))
    }

    pub const fn from_raw(limbs: [u64; 4]) -> Fr {
        Fr(limbs)
    }

    #[inline]
    pub fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Fr([
            u64::conditional_select(&a.0[0], &b.0[0], choice),
            u64::conditional_select(&a.0[1], &b.0[1], choice),
            u64::conditional_select(&a.0[2], &b.0[2], choice),
            u64::conditional_select(&a.0[3], &b.0[3], choice),
        ])
    }
}

impl Add<&Fr> for &Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: &Fr) -> Fr {
        let mut tmp = Fr::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_add(
                &mut tmp.0,
                &self.0,
                &rhs.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            tmp = arithmetic::add(self, rhs);
        }
        tmp
    }
}

impl Add<Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: Fr) -> Fr {
        &self + &rhs
    }
}

impl Add<&Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: &Fr) -> Fr {
        &self + rhs
    }
}

impl AddAssign<Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: Fr) {
        *self = &*self + &rhs;
    }
}

impl AddAssign<&Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: &Fr) {
        *self = &*self + rhs;
    }
}

impl Sub<&Fr> for &Fr {
    type Output = Fr;

    #[inline]
    fn sub(self, rhs: &Fr) -> Fr {
        let mut tmp = Fr::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_sub(
                &mut tmp.0,
                &self.0,
                &rhs.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            tmp = arithmetic::sub(self, rhs);
        }
        tmp
    }
}

impl Sub<Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn sub(self, rhs: Fr) -> Fr {
        &self - &rhs
    }
}

impl Sub<&Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn sub(self, rhs: &Fr) -> Fr {
        &self - rhs
    }
}

impl SubAssign<Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: Fr) {
        *self = &*self - &rhs;
    }
}

impl SubAssign<&Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: &Fr) {
        *self = &*self - rhs;
    }
}

impl Mul<&Fr> for &Fr {
    type Output = Fr;

    #[inline]
    fn mul(self, rhs: &Fr) -> Fr {
        let mut tmp = Fr::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_mul(
                &mut tmp.0,
                &self.0,
                &rhs.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            tmp = arithmetic::mul(self, rhs);
        }
        tmp
    }
}

impl Mul<Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn mul(self, rhs: Fr) -> Fr {
        &self * &rhs
    }
}

impl Mul<&Fr> for Fr {
    type Output = Fr;

    #[inline]
    fn mul(self, rhs: &Fr) -> Fr {
        &self * rhs
    }
}

impl MulAssign<Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: Fr) {
        *self = &*self * &rhs;
    }
}

impl MulAssign<&Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: &Fr) {
        *self = &*self * rhs;
    }
}

impl Neg for &Fr {
    type Output = Fr;

    #[inline]
    fn neg(self) -> Fr {
        let mut tmp = Fr::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_neg(
                &mut tmp.0,
                &self.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            tmp = arithmetic::neg(self);
        }
        tmp
    }
}

impl Neg for Fr {
    type Output = Fr;

    #[inline]
    fn neg(self) -> Fr {
        -&self
    }
}

impl Field for Fr {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();

    fn random(mut rng: impl RngCore) -> Self {
        let mut bytes = [0u8; 32];
        loop {
            rng.fill_bytes(&mut bytes);
            if let Some(fr) = Self::from_bytes(&bytes).into() {
                return fr;
            }
        }
    }

    fn square(&self) -> Self {
        let mut tmp = Self::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_square(
                &mut tmp.0,
                &self.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            tmp = arithmetic::square(self);
        }
        tmp
    }

    fn double(&self) -> Self {
        self + self
    }

    fn invert(&self) -> CtOption<Self> {
        let mut tmp = Self::zero();
        #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_inv(
                &mut tmp.0,
                &self.0,
            );
        }
        #[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
        {
            return arithmetic::invert(self);
        }
        CtOption::new(tmp, !self.ct_eq(&Self::zero()))
    }

    fn sqrt_ratio(_: &Self, _: &Self) -> (Choice, Self) {
        (Choice::from(1u8), Self::one())
    }
}

impl PrimeField for Fr {
    type Repr = [u8; 32];

    const MODULUS: &'static str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
    const NUM_BITS: u32 = 254;
    const CAPACITY: u32 = 253;
    const TWO_INV: Self = Fr([0x7f80000000000001, 0xb784000000000001, 0x0, 0x0]);
    
    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        Self::from_bytes(&repr)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut res = [0u8; 32];
        for i in 0..4 {
            res[i*8..(i+1)*8].copy_from_slice(&self.0[i].to_le_bytes());
        }
        res
    }

    fn is_odd(&self) -> Choice {
        Choice::from((self.0[0] & 1) as u8)
    }
}

impl ConditionallySelectable for Fr {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Fr::conditional_select(a, b, choice)
    }
}

impl ConstantTimeEq for Fr {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl Sum for Fr {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl<'a> Sum<&'a Fr> for Fr {
    fn sum<I: Iterator<Item = &'a Fr>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl Product for Fr {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

impl<'a> Product<&'a Fr> for Fr {
    fn product<I: Iterator<Item = &'a Fr>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

impl Debug for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fr({:?})", self.0)
    }
}

impl Display for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let a = Fr::one();
        let b = Fr::one();
        let c = &a + &b;
        assert_eq!(c, Fr([2, 0, 0, 0]));

        let d = &c * &b;
        assert_eq!(d, Fr([2, 0, 0, 0]));

        let e = -&d;
        assert_ne!(e, d);
    }
}