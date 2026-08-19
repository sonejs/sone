//! Small conversion traits, so a builder call takes the number you have.

/// Anything that can stand in for an IR number.
///
/// The IR is `f32` throughout, but writing `.gap(20.0)` everywhere is noise —
/// an integer literal has no reason to be rejected here.
pub trait Num: Copy {
    fn as_f32(self) -> f32;
}

macro_rules! impl_num {
    ($($ty:ty),+ $(,)?) => {$(
        impl Num for $ty {
            fn as_f32(self) -> f32 {
                self as f32
            }
        }
    )+};
}

impl_num!(f32, f64, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// Anything that can stand in for an IR integer, for counts and spans.
pub trait Int: Copy {
    fn as_i64(self) -> i64;
}

macro_rules! impl_int {
    ($($ty:ty),+ $(,)?) => {$(
        impl Int for $ty {
            fn as_i64(self) -> i64 {
                self as i64
            }
        }
    )+};
}

impl_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// Corner radii: one value, or up to four clockwise from the top left.
pub trait Radii {
    fn into_vec(self) -> Vec<f32>;
}

impl<T: Num> Radii for T {
    fn into_vec(self) -> Vec<f32> {
        vec![self.as_f32()]
    }
}

impl<T: Num> Radii for [T; 2] {
    fn into_vec(self) -> Vec<f32> {
        self.iter().map(|value| value.as_f32()).collect()
    }
}

impl<T: Num> Radii for [T; 3] {
    fn into_vec(self) -> Vec<f32> {
        self.iter().map(|value| value.as_f32()).collect()
    }
}

impl<T: Num> Radii for [T; 4] {
    fn into_vec(self) -> Vec<f32> {
        self.iter().map(|value| value.as_f32()).collect()
    }
}

/// Formats a number the way CSS wants it: no trailing `.0`.
pub(crate) fn css(value: f32) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}
