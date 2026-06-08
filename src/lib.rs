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

macro_rules! impl_byte_conversion {
    ($x:ty) => (
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

/// Similar to `num_traits` `as_()` methods, but works in reverse too.
pub trait LossyCast<T> {
    fn _as(self) -> T;
    fn _from(v: T) -> Self;
}

macro_rules! impl_cast {
    ($x:ty, $y:ty) => (
        impl LossyCast<$y> for $x {
            fn _as(self) -> $y {
                self as _
            }
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
