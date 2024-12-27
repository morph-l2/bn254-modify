use ff::Field;

#[macro_export]
macro_rules! impl_add_binop_specify_output {
    ($lhs:ty, $rhs:ty, $output:ty) => {
        impl Add<$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn add(self, rhs: $rhs) -> $output {
                (&self).add(&rhs)
            }
        }

        impl Add<&$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn add(self, rhs: &$rhs) -> $output {
                (&self).add(rhs)
            }
        }

        impl<'a> Add<$rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn add(self, rhs: $rhs) -> $output {
                self.add(&rhs)
            }
        }

        impl<'a, 'b> Add<&'b $rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn add(self, rhs: &'b $rhs) -> $output {
                let mut result = (*self).clone();
                result += rhs;
                result
            }
        }
    };
}

#[macro_export]
macro_rules! impl_sub_binop_specify_output {
    ($lhs:ty, $rhs:ty, $output:ty) => {
        impl Sub<$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn sub(self, rhs: $rhs) -> $output {
                (&self).sub(&rhs)
            }
        }

        impl Sub<&$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn sub(self, rhs: &$rhs) -> $output {
                (&self).sub(rhs)
            }
        }

        impl<'a> Sub<$rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn sub(self, rhs: $rhs) -> $output {
                self.sub(&rhs)
            }
        }

        impl<'a, 'b> Sub<&'b $rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn sub(self, rhs: &'b $rhs) -> $output {
                let mut result = (*self).clone();
                result -= rhs;
                result
            }
        }
    };
}

#[macro_export]
macro_rules! impl_binops_multiplicative_mixed {
    ($lhs:ty, $rhs:ty, $output:ty) => {
        impl Mul<$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn mul(self, rhs: $rhs) -> $output {
                (&self).mul(&rhs)
            }
        }

        impl Mul<&$rhs> for $lhs {
            type Output = $output;
            #[inline]
            fn mul(self, rhs: &$rhs) -> $output {
                (&self).mul(rhs)
            }
        }

        impl<'a> Mul<$rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn mul(self, rhs: $rhs) -> $output {
                self.mul(&rhs)
            }
        }

        impl<'a, 'b> Mul<&'b $rhs> for &'a $lhs {
            type Output = $output;
            #[inline]
            fn mul(self, rhs: &'b $rhs) -> $output {
                let mut result = (*self).clone();
                result *= rhs;
                result
            }
        }
    };
}

#[macro_export]
macro_rules! impl_sum_prod {
    ($type:ty) => {
        impl ::core::iter::Sum for $type {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(<$type as Field>::ZERO, |acc, item| acc + item)
            }
        }

        impl<'a> ::core::iter::Sum<&'a $type> for $type {
            fn sum<I: Iterator<Item = &'a $type>>(iter: I) -> Self {
                iter.fold(<$type as Field>::ZERO, |acc, item| acc + *item)
            }
        }

        impl ::core::iter::Product for $type {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(<$type as Field>::ONE, |acc, item| acc * item)
            }
        }

        impl<'a> ::core::iter::Product<&'a $type> for $type {
            fn product<I: Iterator<Item = &'a $type>>(iter: I) -> Self {
                iter.fold(<$type as Field>::ONE, |acc, item| acc * *item)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_binops_additive_specify_output {
    ($lhs:ty, $rhs:ty, $output:ty) => {
        impl_add_binop_specify_output!($lhs, $rhs, $output);
        impl_sub_binop_specify_output!($lhs, $rhs, $output);
    };
}