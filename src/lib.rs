use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use std::sync::Once;

use env_logger::Env;

static INIT_TRACING: Once = Once::new();

pub fn init_tracing() {
    INIT_TRACING.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    });
}

// pub mod ext;
// pub mod generated;
pub use numutil_proc_macro::make_generic_num;
// make_generic_num!(GenericNumContainerName, (u8, u16), hello);

#[cfg(test)]
mod test {
    use super::*;
    use numutil_proc_macro::make_generic_num;
    make_generic_num!(GenericNum, (u8, u16, f32), (Num), (Debug, Clone));
    #[test]
    fn test_derived_container() {
        init_tracing();
        let mut nv = GenericNumVec::U8(vec![0, 1, 2, 3]);
        let nv_f32: Vec<f32> = nv.num_cast();
        nv.set(2, 5.0.as_generic_num());
        log::info!("{nv:#?} {nv_f32:#?} {:?}", nv_f32.get(2));
    }
}

pub trait Num:
    std::fmt::Display
    + std::fmt::Debug
    + ConstOne
    + ConstZero
    + UnsizedByteConversion
    + Copy
    + Sized
    + LossyFromPrimitive
    + LossyUnsizedFromPrimitive
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Rem<Self, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + RemAssign<Self>
    // + Neg
    + PartialOrd
    + PartialEq // + Ord
{
    fn max(&self, other: &Self) -> Self {
        match self.partial_cmp(other) {
            Some(Ordering::Greater) => *self,
            _ => *other,
        }
    }
    fn min(&self, other: &Self) -> Self {
        match self.partial_cmp(other) {
            Some(Ordering::Less) => *self,
            _ => *other,
        }
    }
    fn eq(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}
impl<
    T: std::fmt::Display
        + std::fmt::Debug
        + ConstOne
        + ConstZero
        + UnsizedByteConversion
        + Copy
        + Sized
        + LossyFromPrimitive
        + Div<Self, Output = Self>
        + Mul<Self, Output = Self>
        + Sub<Self, Output = Self>
        + Add<Self, Output = Self>
        + Rem<Self, Output = Self>
        + AddAssign<Self>
        + SubAssign<Self>
        + MulAssign<Self>
        + DivAssign<Self>
        + RemAssign<Self>
        // + Neg
        + PartialOrd
        + PartialEq, // + Ord,
> Num for T
where
    Vec<T>: VecByteConversion,
{
}

pub trait LossyFromPrimitive:
    LossyCast<usize>
    + LossyCast<usize>
    + LossyCast<isize>
    + LossyCast<u8>
    + LossyCast<i8>
    + LossyCast<u16>
    + LossyCast<i16>
    + LossyCast<u32>
    + LossyCast<i32>
    + LossyCast<u64>
    + LossyCast<i64>
    + LossyCast<u128>
    + LossyCast<i128>
    + LossyCast<f32>
    + LossyCast<f64>
{
}
impl<
    T: LossyCast<usize>
        + LossyCast<usize>
        + LossyCast<isize>
        + LossyCast<u8>
        + LossyCast<i8>
        + LossyCast<u16>
        + LossyCast<i16>
        + LossyCast<u32>
        + LossyCast<i32>
        + LossyCast<u64>
        + LossyCast<i64>
        + LossyCast<u128>
        + LossyCast<i128>
        + LossyCast<f32>
        + LossyCast<f64>,
> LossyFromPrimitive for T
{
}

pub trait LossyUnsizedFromPrimitive:
    std::fmt::Debug
    + std::fmt::Display
    + LossyCastUnsized<usize>
    + LossyCastUnsized<usize>
    + LossyCastUnsized<isize>
    + LossyCastUnsized<u8>
    + LossyCastUnsized<i8>
    + LossyCastUnsized<u16>
    + LossyCastUnsized<i16>
    + LossyCastUnsized<u32>
    + LossyCastUnsized<i32>
    + LossyCastUnsized<u64>
    + LossyCastUnsized<i64>
    + LossyCastUnsized<u128>
    + LossyCastUnsized<i128>
    + LossyCastUnsized<f32>
    + LossyCastUnsized<f64>
{
}
impl<
    T: std::fmt::Debug
        + std::fmt::Display
        + LossyCastUnsized<usize>
        + LossyCastUnsized<usize>
        + LossyCastUnsized<isize>
        + LossyCastUnsized<u8>
        + LossyCastUnsized<i8>
        + LossyCastUnsized<u16>
        + LossyCastUnsized<i16>
        + LossyCastUnsized<u32>
        + LossyCastUnsized<i32>
        + LossyCastUnsized<u64>
        + LossyCastUnsized<i64>
        + LossyCastUnsized<u128>
        + LossyCastUnsized<i128>
        + LossyCastUnsized<f32>
        + LossyCastUnsized<f64>,
> LossyUnsizedFromPrimitive for T
{
}

pub trait ConstOne {
    const ONE: Self;
}
pub trait ConstZero {
    const ZERO: Self;
}

macro_rules! impl_common_const {
    ($x:ty) => (
        impl ConstZero for $x {
            const ZERO: $x = 0 as $x;
        }
        impl ConstOne for $x {
            const ONE: $x = 1 as $x;
        }
    );
    ($x:ty, $($y:ty),+) => (
        impl_common_const!($x);
        impl_common_const!($($y),+);
    )
}
impl_common_const!(
    usize, isize, i8, u8, i16, i32, i64, i128, u16, u32, u64, u128, f32, f64
);

/// Methods for converting between arrays of `T` and bytes, assuming `T` implements `to_(n/l/b)e_bytes()`.
pub trait VecByteConversion
where
    Self: Sized,
{
    fn to_le_bytes(&self) -> Vec<u8>;
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
    fn to_be_bytes(&self) -> Vec<u8>;
    fn from_be_bytes(bytes: &[u8]) -> Option<Self>;
    fn to_ne_bytes(&self) -> Vec<u8>;
    fn from_ne_bytes(bytes: &[u8]) -> Option<Self>;
}

/// Trait useful for accepting any type `T` that implements `(to/from)_(n/l/b)e_bytes()` as a function input.
pub trait ByteConversion<const N: usize> {
    fn to_le_bytes_(self) -> [u8; N];
    fn from_le_bytes_(bytes: [u8; N]) -> Self;
    fn to_be_bytes_(self) -> [u8; N];
    fn from_be_bytes_(bytes: [u8; N]) -> Self;
    fn to_ne_bytes_(self) -> [u8; N];
    fn from_ne_bytes_(bytes: [u8; N]) -> Self;
}
// pub trait ByteConversionAlt {
//     fn to_le_bytes_(self) -> &'_ [u8];
//     fn from_le_bytes_(bytes: &[u8]) -> Self;
//     fn to_be_bytes_(self) -> &'_ [u8];
//     fn from_be_bytes_(bytes: &[u8]) -> Self;
//     fn to_ne_bytes_(self) -> &'_ [u8];
//     fn from_ne_bytes_(bytes: &[u8]) -> Self;
// }
/// Trait useful for accepting any type `T` that implements `(to/from)_(n/l/b)e_bytes()` as a function input. Requires heap alloc, so `VecByteConversion` is preferred.
pub trait UnsizedByteConversion: Sized {
    fn to_le_bytes_(self) -> Box<[u8]>;
    fn from_le_bytes_(bytes: &[u8]) -> Option<Self>;
    fn to_be_bytes_(self) -> Box<[u8]>;
    fn from_be_bytes_(bytes: &[u8]) -> Option<Self>;
    fn to_ne_bytes_(self) -> Box<[u8]>;
    fn from_ne_bytes_(bytes: &[u8]) -> Option<Self>;
}

macro_rules! impl_byte_conversion {
    ($x:ty) => (
        //  impl ByteConversionAlt for $x {
        //     fn to_le_bytes_(self) -> &'_ [u8] {
        //         &<$x>::to_le_bytes(self)
        //     }

        //     fn from_le_bytes_(bytes: &[u8]) -> Self {
        //         <$x>::from_le_bytes(bytes.try_into().unwrap())
        //     }

        //     fn to_be_bytes_(self) -> &'_ [u8] {
        //         &<$x>::to_be_bytes(self)
        //     }

        //     fn from_be_bytes_(bytes: &[u8]) -> Self {
        //         <$x>::from_be_bytes(bytes.try_into().unwrap())
        //     }

        //     fn to_ne_bytes_(self) -> &'_ [u8] {
        //         &<$x>::to_ne_bytes(self)
        //     }

        //     fn from_ne_bytes_(bytes: &[u8]) -> Self {
        //         <$x>::from_ne_bytes(bytes.try_into().unwrap())
        //     }
        // }
        impl ByteConversion<{ size_of::<$x>() }> for $x {
            fn to_le_bytes_(self) -> [u8; size_of::<$x>()] {
                <$x>::to_le_bytes(self)
            }

            fn from_le_bytes_(bytes: [u8; size_of::<$x>()]) -> Self {
                <$x>::from_le_bytes(bytes)
            }

            fn to_be_bytes_(self) -> [u8; size_of::<$x>()] {
                <$x>::to_be_bytes(self)
            }

            fn from_be_bytes_(bytes: [u8; size_of::<$x>()]) -> Self {
                <$x>::from_be_bytes(bytes)
            }

            fn to_ne_bytes_(self) -> [u8; size_of::<$x>()] {
                <$x>::to_ne_bytes(self)
            }

            fn from_ne_bytes_(bytes: [u8; size_of::<$x>()]) -> Self {
                <$x>::from_ne_bytes(bytes)
            }
        }
        impl UnsizedByteConversion for $x {
            fn to_le_bytes_(self) -> Box<[u8]> {
                Box::new(<$x>::to_le_bytes(self))
            }

            fn from_le_bytes_(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != size_of::<$x>() {
                    panic!("Byte count mismatch, expected {}, got {}.", size_of::<$x>(), bytes.len());
                }
                // <$x>::from_le_bytes(unsafe {
                //     (bytes.as_ptr() as *const [u8; size_of::<$x>()]).read()
                // })
                Some(<$x>::from_le_bytes(
                    bytes.try_into().ok()?
                ))
            }

            fn to_be_bytes_(self) -> Box<[u8]> {
                Box::new(<$x>::to_be_bytes(self))
            }

            fn from_be_bytes_(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != size_of::<$x>() {
                    panic!("Byte count mismatch, expected {}, got {}.", size_of::<$x>(), bytes.len());
                }
                // <$x>::from_be_bytes(unsafe {
                //     (bytes.as_ptr() as *const [u8; size_of::<$x>()]).read()
                // })
                Some(<$x>::from_be_bytes(
                    bytes.try_into().ok()?
                ))
            }

            fn to_ne_bytes_(self) -> Box<[u8]> {
                Box::new(<$x>::to_ne_bytes(self))
            }

            fn from_ne_bytes_(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != size_of::<$x>() {
                    panic!("Byte count mismatch, expected {}, got {}.", size_of::<$x>(), bytes.len());
                }
                Some(<$x>::from_ne_bytes(
                    bytes.try_into().ok()?
                ))
            }
        }
        impl VecByteConversion for Vec<$x> {
            fn to_le_bytes(&self) -> Vec<u8> {
                self.iter().flat_map(|v| v.to_le_bytes()).collect()
            }

            fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len().is_multiple_of(size_of::<$x>()) {
                    Some(
                        bytes
                            .chunks_exact(size_of::<$x>())
                            .map(|v| {
                                <$x>::from_le_bytes(unsafe {
                                    (v.as_ptr() as *const [u8; size_of::<$x>()]).read()
                                })
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            }
            fn to_be_bytes(&self) -> Vec<u8> {
                self.iter().flat_map(|v| v.to_be_bytes()).collect()
            }

            fn from_be_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len().is_multiple_of(size_of::<$x>()) {
                    Some(
                        bytes
                            .chunks_exact(size_of::<$x>())
                            .map(|v| {
                                <$x>::from_be_bytes(unsafe {
                                    (v.as_ptr() as *const [u8; size_of::<$x>()]).read()
                                })
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            }
            fn to_ne_bytes(&self) -> Vec<u8> {
                self.iter().flat_map(|v| v.to_ne_bytes()).collect()
            }

            fn from_ne_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len().is_multiple_of(size_of::<$x>()) {
                    Some(
                        bytes
                            .chunks_exact(size_of::<$x>())
                            .map(|v| {
                                <$x>::from_ne_bytes(unsafe {
                                    (v.as_ptr() as *const [u8; size_of::<$x>()]).read()
                                })
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            }
        }
    );
    ($x:ty, $($y:ty),+) => (
        impl_byte_conversion!($x);
        impl_byte_conversion!($($y),+);
    )
}

impl_byte_conversion!(
    usize, isize, i8, u8, i16, i32, i64, i128, u16, u32, u64, u128, f32, f64
);
// : Debug + Display
pub trait LossyCastUnsized<T> {
    fn _as(self) -> T;
}

/// Similar to `num_traits` `as_()` methods, but works in reverse too.
pub trait LossyCast<T>: LossyCastUnsized<T> {
    fn _from(v: T) -> Self;
    fn vec_as(v: &[Self]) -> Vec<T>
    where
        Self: Sized + Copy,
    {
        v.iter().map(|v| v._as()).collect()
    }
    fn vec_from(v: &[T]) -> Vec<Self>
    where
        Self: Sized + Copy,
        T: Copy,
    {
        v.iter().map(|v| Self::_from(*v)).collect()
    }
}

macro_rules! impl_cast {
    ($x:ty, $y:ty) => (
        impl LossyCastUnsized<$y> for $x {
            fn _as(self) -> $y {
                self as _
            }
        }
        impl LossyCast<$y> for $x {
            // fn _as(self) -> $y {
            //     self as _
            // }
            // fn _as(self) -> $y {
            //     self as _
            // }
            fn _from(v: $y) -> $x {
                v as _
            }
        }
    );
    ($x:ty,$y:ty, $($z:ty),+) => (
        impl_cast!($x, $y);
        impl_cast!($x, $($z),+);
    )
}

impl_cast!(
    usize, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    isize, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    i8, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    u8, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    i16, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    i32, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    i64, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    u16, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    u32, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    u64, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    f32, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    f64, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    i128, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);
impl_cast!(
    u128, usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
);

#[cfg(test)]
mod tests {
    use crate::VecByteConversion;

    macro_rules! impl_vec_test {
        ($x:ty) => (
                let src_vec = vec![<$x>::MAX/(4 as $x), <$x>::MAX/(2 as $x),<$x>::MAX/(4 as $x)*(3 as $x),<$x>::MAX];
                let round_trip_vec = Vec::<$x>::from_le_bytes(&src_vec.to_le_bytes()).unwrap();
                for idx in 0..4 {
                    assert_eq!(
                        src_vec[idx],
                        Vec::<$x>::from_le_bytes(&src_vec.to_le_bytes()).unwrap()[idx]
                    );
                }
        );
        ($x:ty, $($y:ty),+) => (
            impl_vec_test!($x);
            impl_vec_test!($($y),+);
        )
    }

    #[test]
    fn test_roundtrip() {
        impl_vec_test!(
            usize, isize, i8, u8, i16, i32, i64, u16, u32, u64, i128, u128, f32, f64
        );
    }
}
