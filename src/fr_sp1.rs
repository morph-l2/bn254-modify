use super::ff::*;
use core::fmt::{self, Debug, Display};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// The BN254 scalar field Fr.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Fr(pub [u32; 8]);

impl Fr {
    pub const fn zero() -> Self {
        Fr([0, 0, 0, 0, 0, 0, 0, 0])
    }

    pub const fn one() -> Self {
        Fr([1, 0, 0, 0, 0, 0, 0, 0])
    }

    pub const fn from_raw(val: [u64; 4]) -> Self {
        let mut tmp = [0u32; 8];
        tmp[0] = val[0] as u32;
        tmp[1] = (val[0] >> 32) as u32;
        tmp[2] = val[1] as u32;
        tmp[3] = (val[1] >> 32) as u32;
        tmp[4] = val[2] as u32;
        tmp[5] = (val[2] >> 32) as u32;
        tmp[6] = val[3] as u32;
        tmp[7] = (val[3] >> 32) as u32;
        Fr(tmp)
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

impl Add for Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: Fr) -> Fr {
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

impl Sub for Fr {
    type Output = Fr;

    #[inline]
    fn sub(self, rhs: Fr) -> Fr {
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

impl Mul for Fr {
    type Output = Fr;

    #[inline]
    fn mul(self, rhs: Fr) -> Fr {
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

impl<'a> Add<&'a Fr> for Fr {
    type Output = Fr;
    fn add(self, rhs: &'a Fr) -> Fr {
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

impl<'a> Sub<&'a Fr> for Fr {
    type Output = Fr;
    fn sub(self, rhs: &'a Fr) -> Fr {
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

impl<'a> Mul<&'a Fr> for Fr {
    type Output = Fr;
    fn mul(self, rhs: &'a Fr) -> Fr {
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

impl AddAssign<&Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: &Fr) {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &Fr::one().0,
                &self.0,
                &rhs.0,
            );
        }
        *self = tmp;
    }
}

impl SubAssign<&Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: &Fr) {
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
        *self = tmp;
    }
}

impl MulAssign<&Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: &Fr) {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &self.0,
                &rhs.0,
                &Fr::zero().0,
            );
        }
        *self = tmp;
    }
}

impl AddAssign<Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: Fr) {
        self.add_assign(&rhs)
    }
}

impl SubAssign<Fr> for Fr {
    #[inline]
    fn sub_assign(&mut self, rhs: Fr) {
        self.sub_assign(&rhs)
    }
}

impl MulAssign<Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: Fr) {
        self.mul_assign(&rhs)
    }
}

impl Sum for Fr {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Fr::zero();
        for x in iter {
            acc.add_assign(&x);
        }
        acc
    }
}

impl<'a> Sum<&'a Fr> for Fr {
    fn sum<I: Iterator<Item = &'a Fr>>(iter: I) -> Self {
        let mut acc = Fr::zero();
        for x in iter {
            acc.add_assign(x);
        }
        acc
    }
}

impl Product for Fr {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut acc = Fr::one();
        for x in iter {
            acc.mul_assign(&x);
        }
        acc
    }
}

impl<'a> Product<&'a Fr> for Fr {
    fn product<I: Iterator<Item = &'a Fr>>(iter: I) -> Self {
        let mut acc = Fr::one();
        for x in iter {
            acc.mul_assign(x);
        }
        acc
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

impl Field for Fr {
    fn random(mut rng: impl RngCore) -> Self {
        let mut repr = [0u32; 8];
        for r in repr.iter_mut() {
            *r = rng.next_u32();
        }
        Fr(repr)
    }

    fn zero() -> Self {
        Fr::zero()
    }

    fn one() -> Self {
        Fr::one()
    }

    fn square(&self) -> Self {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &self.0,
                &self.0,
                &Fr::zero().0,
            );
        }
        tmp
    }

    fn double(&self) -> Self {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &[2, 0, 0, 0, 0, 0, 0, 0],
                &self.0,
                &Fr::zero().0,
            );
        }
        tmp
    }

    fn invert(&self) -> CtOption<Self> {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_muladd(
                &mut tmp.0,
                &[0xFFFFFFFF; 8],
                &self.0,
                &Fr::zero().0,
            );
        }
        CtOption::new(tmp, !self.ct_eq(&Fr::zero()))
    }

    fn sqrt_ratio(_: &Self, _: &Self) -> (Choice, Self) {
        // TODO: Implement proper sqrt_ratio
        (Choice::from(1u8), Fr::one())
    }
}

impl PrimeField for Fr {
    type Repr = [u8; 32];

    const MODULUS: &'static str = "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001";
    const NUM_BITS: u32 = 254;
    const CAPACITY: u32 = 253;
    
    const TWO_INV: Self = Fr::from_raw([
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]);

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
        let mut tmp = [0u32; 8];
        for i in 0..8 {
            let mut val = 0u32;
            for j in 0..4 {
                val |= (repr[i * 4 + j] as u32) << (j * 8);
            }
            tmp[i] = val;
        }
        CtOption::new(Fr(tmp), Choice::from(1u8))
    }

    fn to_repr(&self) -> Self::Repr {
        let mut res = [0u8; 32];
        for i in 0..8 {
            for j in 0..4 {
                res[i * 4 + j] = (self.0[i] >> (j * 8)) as u8;
            }
        }
        res
    }

    fn is_odd(&self) -> Choice {
        Choice::from((self.0[0] & 1) as u8)
    }
}

impl Debug for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fr([")
            .and_then(|_| {
                for (i, val) in self.0.iter().enumerate() {
                    if i == self.0.len() - 1 {
                        write!(f, "{:#X}", val)?;
                    } else {
                        write!(f, "{:#X}, ", val)?;
                    }
                }
                Ok(())
            })
            .and_then(|_| write!(f, "])"))
    }
}

impl Display for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<u64> for Fr {
    fn from(val: u64) -> Self {
        let mut repr = [0u32; 8];
        repr[0] = val as u32;
        repr[1] = (val >> 32) as u32;
        Fr(repr)
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
}