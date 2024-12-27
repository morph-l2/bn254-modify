use super::ff::*;
use core::fmt::{self, Debug, Display};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

const MODULUS: Fr = Fr([
    0x43E1F593,
    0x2833E848,
    0x81585D2,
    0xB85045B6,
    0xE131A029,
    0x30644E72,
    0x0,
    0x0,
]);

#[inline(always)]
const fn sbb_u32(a: u32, b: u32, borrow: u32) -> (u32, u32) {
    let ret = (a as u64).wrapping_sub((b as u64) + (borrow as u64));
    (ret as u32, (ret >> 32) as u32)
}

/// The BN254 scalar field Fr.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Fr(pub [u32; 8]);

impl Fr {
    #[inline]
    pub const fn zero() -> Self {
        Fr([0, 0, 0, 0, 0, 0, 0, 0])
    }

    #[inline]
    pub const fn one() -> Self {
        Fr([1, 0, 0, 0, 0, 0, 0, 0])
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Fr> {
        let mut tmp = [0, 0, 0, 0, 0, 0, 0, 0];

        tmp[0] = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        tmp[1] = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        tmp[2] = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        tmp[3] = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        tmp[4] = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        tmp[5] = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        tmp[6] = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        tmp[7] = u32::from_le_bytes(bytes[28..32].try_into().unwrap());

        let (_, borrow) = sbb_u32(tmp[0], MODULUS.0[0], 0);
        let (_, borrow) = sbb_u32(tmp[1], MODULUS.0[1], borrow);
        let (_, borrow) = sbb_u32(tmp[2], MODULUS.0[2], borrow);
        let (_, borrow) = sbb_u32(tmp[3], MODULUS.0[3], borrow);
        let (_, borrow) = sbb_u32(tmp[4], MODULUS.0[4], borrow);
        let (_, borrow) = sbb_u32(tmp[5], MODULUS.0[5], borrow);
        let (_, borrow) = sbb_u32(tmp[6], MODULUS.0[6], borrow);
        let (_, borrow) = sbb_u32(tmp[7], MODULUS.0[7], borrow);

        let is_some = (borrow as u8) & 1;

        CtOption::new(Fr(tmp), Choice::from(is_some))
    }

    pub const fn from_raw(limbs: [u64; 4]) -> Fr {
        let mut tmp = [0, 0, 0, 0, 0, 0, 0, 0];

        tmp[0] = (limbs[0] & 0xffffffff) as u32;
        tmp[1] = ((limbs[0] >> 32) & 0xffffffff) as u32;
        tmp[2] = (limbs[1] & 0xffffffff) as u32;
        tmp[3] = ((limbs[1] >> 32) & 0xffffffff) as u32;
        tmp[4] = (limbs[2] & 0xffffffff) as u32;
        tmp[5] = ((limbs[2] >> 32) & 0xffffffff) as u32;
        tmp[6] = (limbs[3] & 0xffffffff) as u32;
        tmp[7] = ((limbs[3] >> 32) & 0xffffffff) as u32;

        Fr(tmp)
    }

    pub const fn size() -> usize {
        32
    }

    #[inline]
    pub fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut res = Self::zero();
        for i in 0..8 {
            res.0[i] = u32::conditional_select(&a.0[i], &b.0[i], choice);
        }
        res
    }
}

impl Add<&Fr> for &Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: &Fr) -> Fr {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &Fr::one().0,
                &self.0,
                &rhs.0,
            );
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
        let neg_rhs = -rhs;
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &Fr::one().0,
                &self.0,
                &neg_rhs.0,
            );
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
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &self.0,
                &rhs.0,
                &Fr::zero().0,
            );
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
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &[0xFFFFFFFF; 8],
                &self.0,
                &Fr::zero().0,
            );
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
        let mut buf = [0u8; 32];
        loop {
            rng.fill_bytes(&mut buf);
            if let Some(fr) = Self::from_bytes(&buf).into() {
                return fr;
            }
        }
    }

    fn square(&self) -> Self {
        let mut tmp = Self::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &self.0,
                &self.0,
                &Self::zero().0,
            );
        }
        tmp
    }

    fn double(&self) -> Self {
        let mut tmp = Self::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &[2, 0, 0, 0, 0, 0, 0, 0],
                &self.0,
                &Self::zero().0,
            );
        }
        tmp
    }

    fn invert(&self) -> CtOption<Self> {
        let mut tmp = Self::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &[0xFFFFFFFF; 8],
                &self.0,
                &Self::zero().0,
            );
        }
        CtOption::new(tmp, !self.ct_eq(&Self::zero()))
    }

    fn sqrt_ratio(_: &Self, _: &Self) -> (Choice, Self) {
        (Choice::from(1u8), Self::one())
    }
}

impl PrimeField for Fr {
    type Repr = [u8; 32];

    const MODULUS: &'static str = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";
    const NUM_BITS: u32 = 254;
    const CAPACITY: u32 = 253;
    
    const TWO_INV: Self = Fr::from_raw([2, 0, 0, 0]);
    const MULTIPLICATIVE_GENERATOR: Self = Fr::from_raw([2, 0, 0, 0]);
    
    const S: u32 = 28;
    
    const ROOT_OF_UNITY: Self = Fr::from_raw([
        0xd35d438dc58f0d9d,
        0x0a78eb28f5c70b3d,
        0x666ea36f7879462c,
        0x0e0a77c19a07df2f,
    ]);
    
    const ROOT_OF_UNITY_INV: Self = Fr::from_raw([
        0x4dca135978a8016a,
        0x0559f96e8c0d3308,
        0xc6695f92b50a8313,
        0x12c1b6aaf3dd5b7b,
    ]);
    
    const DELTA: Self = Fr::from_raw([
        0x94575516c2c9b3b2,
        0x00000001236392ee,
        0x0000000000000001,
        0x0000000000000000,
    ]);

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        Self::from_bytes(&repr)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut res = [0u8; 32];
        for i in 0..8 {
            res[i*4..(i+1)*4].copy_from_slice(&self.0[i].to_le_bytes());
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
        let mut tmp = 0u8;
        for i in 0..8 {
            tmp |= (self.0[i] ^ other.0[i]) as u8;
        }
        Choice::from((tmp == 0) as u8)
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
        write!(f, "Fr([{:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}])",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5], self.0[6], self.0[7])
    }
}

impl Display for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<u64> for Fr {
    fn from(val: u64) -> Self {
        let mut tmp = [0u32; 8];
        tmp[0] = val as u32;
        tmp[1] = (val >> 32) as u32;
        Fr(tmp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fr_from_raw() {
        let raw = [1u64, 2u64, 3u64, 4u64];
        let fr = Fr::from_raw(raw);
        
        assert_eq!(fr.0[0], 1u32);
        assert_eq!(fr.0[1], 0u32);
        assert_eq!(fr.0[2], 2u32);
        assert_eq!(fr.0[3], 0u32);
        assert_eq!(fr.0[4], 3u32);
        assert_eq!(fr.0[5], 0u32);
        assert_eq!(fr.0[6], 4u32);
        assert_eq!(fr.0[7], 0u32);
    }

    #[test]
    fn test_fr_basic_ops() {
        let a = Fr::from_raw([1, 0, 0, 0]);
        let b = Fr::from_raw([2, 0, 0, 0]);
        
        let c = &a + &b;
        assert_eq!(c.0[0], 3);
        
        let d = &c - &b;
        assert_eq!(d.0[0], 1);
        
        let e = &a * &b;
        assert_eq!(e.0[0], 2);
    }
}