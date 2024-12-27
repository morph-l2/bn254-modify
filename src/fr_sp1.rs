use super::ff::*;
use core::fmt::{self, Debug, Display};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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

    /// Creates a new field element from raw u64 values.
    /// The values should be in little-endian order.
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
}

impl Add for Fr {
    type Output = Fr;

    #[inline]
    fn add(self, rhs: Fr) -> Fr {
        let mut tmp = self;
        tmp.add_assign(&rhs);
        tmp
    }
}

impl Sub for Fr {
    type Output = Fr;

    #[inline]
    fn sub(self, rhs: Fr) -> Fr {
        let mut tmp = self;
        tmp.sub_assign(&rhs);
        tmp
    }
}

impl Mul for Fr {
    type Output = Fr;

    #[inline]
    fn mul(self, rhs: Fr) -> Fr {
        let mut tmp = self;
        tmp.mul_assign(&rhs);
        tmp
    }
}

impl AddAssign<&Fr> for Fr {
    #[inline]
    fn add_assign(&mut self, rhs: &Fr) {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_mac(&mut tmp.0, &rhs.0, &Fr::one().0, &self.0);
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
            sp1_intrinsics::bn254::syscall_bn254_scalar_mac(&mut tmp.0, &neg_rhs.0, &Fr::one().0, &self.0);
        }
        *self = tmp;
    }
}

impl MulAssign<&Fr> for Fr {
    #[inline]
    fn mul_assign(&mut self, rhs: &Fr) {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_mul(&mut tmp.0, &self.0, &rhs.0);
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

impl Neg for &Fr {
    type Output = Fr;

    #[inline]
    fn neg(self) -> Fr {
        let mut tmp = Fr::zero();
        unsafe {
            sp1_intrinsics::bn254::syscall_bn254_scalar_mul(&mut tmp.0, &self.0, &[
                0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
                0xFFFFFFFF,
            ]);
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

impl Field for Fr {
    fn random(mut rng: impl RngCore) -> Self {
        let mut repr = [0u32; 8];
        for r in repr.iter_mut() {
            *r = rng.next_u32();
        }
        Fr(repr)
    }

    const ZERO: Self = Fr::zero();
    const ONE: Self = Fr::one();
}

impl PrimeField for Fr {
    fn from_repr(repr: [u8; 32]) -> Option<Self> {
        let mut tmp = [0u32; 8];
        for (i, chunk) in repr.chunks(4).enumerate() {
            let mut val = 0u32;
            for (j, &byte) in chunk.iter().enumerate() {
                val |= (byte as u32) << (j * 8);
            }
            tmp[i] = val;
        }
        Some(Fr(tmp))
    }

    fn to_repr(&self) -> [u8; 32] {
        let mut res = [0u8; 32];
        for (i, &val) in self.0.iter().enumerate() {
            let start = i * 4;
            res[start] = val as u8;
            res[start + 1] = (val >> 8) as u8;
            res[start + 2] = (val >> 16) as u8;
            res[start + 3] = (val >> 24) as u8;
        }
        res
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