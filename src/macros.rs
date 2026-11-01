macro_rules! impl_from_enum_for_int {
    ($enum:ty => $($int:ty),+ $(,)?) => {
        $(
            impl From<$enum> for $int {
                fn from(value: $enum) -> $int {
                    value as $int
                }
            }
        )+
    };
}

macro_rules! impl_try_from_int_for_enum {
    ($enum:ty => $($int:ty),+ $(,)?) => {
        $(
            impl TryFrom<$int> for $enum {
                type Error = core::num::TryFromIntError;

                fn try_from(value: $int) -> Result<Self, Self::Error> {
                    let repr = u8::try_from(value)?;
                    <$enum>::from_u8(repr).ok_or_else(|| {
                        i8::try_from(u8::MAX).unwrap_err()
                    })
                }
            }
        )+
    };
}
