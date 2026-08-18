use proc_macro2::Literal;

pub trait RustType {
    fn as_rust_type(&self) -> &'static str;
}

pub trait CppType {
    fn as_cpp_type(&self) -> &'static str;
}

pub trait RustIntegerLiteral {
    fn literal(&self, value: i64) -> Literal;
}

pub trait RustFloatLiteral {
    fn fliteral(&self, value: f64) -> Literal;
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawType {
    Float32,
    Float64,
    Integer(IntReprType),
}

impl RustType for RawType {
    fn as_rust_type(&self) -> &'static str {
        match self {
            RawType::Float32 => "f32",
            RawType::Float64 => "f64",
            RawType::Integer(v) => v.as_rust_type(),
        }
    }
}

impl CppType for RawType {
    fn as_cpp_type(&self) -> &'static str {
        match self {
            RawType::Float32 => "float",
            RawType::Float64 => "double",
            RawType::Integer(v) => v.as_cpp_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EnumCoverage {
    Exhaustive,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PhysicalType {
    Bool,
    Float32,
    Float64,
    Integer(IntReprType),
    Enum {
        coverage: EnumCoverage,
        repr: IntReprType,
    },
}

impl PhysicalType {
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, PhysicalType::Float32 | PhysicalType::Float64)
    }

    #[must_use]
    pub fn min_value_f64(&self) -> f64 {
        match self {
            PhysicalType::Bool => 0.0,
            PhysicalType::Float32 => f64::from(f32::MIN),
            PhysicalType::Float64 => f64::MIN,
            PhysicalType::Integer(repr) | PhysicalType::Enum { repr, .. } => repr.min_value_f64(),
        }
    }

    #[must_use]
    pub fn max_value_f64(&self) -> f64 {
        match self {
            PhysicalType::Bool => 1.0,
            PhysicalType::Float32 => f64::from(f32::MAX),
            PhysicalType::Float64 => f64::MAX,
            PhysicalType::Integer(repr) | PhysicalType::Enum { repr, .. } => repr.max_value_f64(),
        }
    }

    #[must_use]
    pub fn integer_range_f64(&self) -> Option<(f64, f64)> {
        match self {
            PhysicalType::Integer(repr) | PhysicalType::Enum { repr, .. } => {
                Some((repr.min_value_f64(), repr.max_value_f64()))
            }
            _ => None,
        }
    }
}

impl RustType for PhysicalType {
    fn as_rust_type(&self) -> &'static str {
        match self {
            PhysicalType::Bool => "bool",
            PhysicalType::Float32 => "f32",
            PhysicalType::Float64 => "f64",
            PhysicalType::Integer(v) => v.as_rust_type(),
            PhysicalType::Enum { repr, .. } => repr.as_rust_type(),
        }
    }
}

impl CppType for PhysicalType {
    fn as_cpp_type(&self) -> &'static str {
        match self {
            PhysicalType::Bool => "bool",
            PhysicalType::Float32 => "float",
            PhysicalType::Float64 => "double",
            PhysicalType::Integer(v) => v.as_cpp_type(),
            PhysicalType::Enum { repr, .. } => repr.as_cpp_type(),
        }
    }
}

impl RustIntegerLiteral for PhysicalType {
    fn literal(&self, value: i64) -> Literal {
        match self {
            PhysicalType::Integer(repr) | PhysicalType::Enum { repr, .. } => repr.literal(value),
            _ => panic!("Use only with integer types"),
        }
    }
}

impl RustFloatLiteral for PhysicalType {
    fn fliteral(&self, value: f64) -> Literal {
        match self {
            PhysicalType::Float32 => Literal::f32_suffixed(value as f32),
            PhysicalType::Float64 => Literal::f64_suffixed(value),
            _ => panic!("Use only with floating point types"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntReprType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    #[default]
    I128,
}

impl IntReprType {
    #[must_use]
    pub fn from_size_sign(size: u64, signed: bool) -> IntReprType {
        match (signed, size) {
            (false, 0..=8) => IntReprType::U8,
            (false, 9..=16) => IntReprType::U16,
            (false, 17..=32) => IntReprType::U32,
            (false, 33..=64) => IntReprType::U64,
            (false, _) => IntReprType::U128,

            (true, 0..=8) => IntReprType::I8,
            (true, 9..=16) => IntReprType::I16,
            (true, 17..=32) => IntReprType::I32,
            (true, 33..=64) => IntReprType::I64,
            (true, _) => IntReprType::I128,
        }
    }

    #[must_use]
    pub fn min_value_i64(&self) -> i64 {
        match self {
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128 => 0,
            Self::I8 => i64::from(i8::MIN),
            Self::I16 => i64::from(i16::MIN),
            Self::I32 => i64::from(i32::MIN),
            Self::I64 | Self::I128 => i64::MIN,
        }
    }

    #[must_use]
    pub fn max_value_i64(&self) -> i64 {
        match self {
            Self::U8 => i64::from(u8::MAX),
            Self::U16 => i64::from(u16::MAX),
            Self::U32 => i64::from(u32::MAX),
            Self::U64 | Self::U128 => i64::MAX,
            Self::I8 => i64::from(i8::MAX),
            Self::I16 => i64::from(i16::MAX),
            Self::I32 => i64::from(i32::MAX),
            Self::I64 | Self::I128 => i64::MAX,
        }
    }

    #[must_use]
    pub fn min_value_f64(self) -> f64 {
        match self {
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128 => 0.0,
            Self::I8 => f64::from(i8::MIN),
            Self::I16 => f64::from(i16::MIN),
            Self::I32 => f64::from(i32::MIN),
            Self::I64 => i64::MIN as f64,
            Self::I128 => i128::MIN as f64,
        }
    }

    #[must_use]
    pub fn max_value_f64(self) -> f64 {
        match self {
            Self::U8 => f64::from(u8::MAX),
            Self::U16 => f64::from(u16::MAX),
            Self::U32 => f64::from(u32::MAX),
            Self::U64 => u64::MAX as f64,
            Self::U128 => u128::MAX as f64,
            Self::I8 => f64::from(i8::MAX),
            Self::I16 => f64::from(i16::MAX),
            Self::I32 => f64::from(i32::MAX),
            Self::I64 => i64::MAX as f64,
            Self::I128 => i128::MAX as f64,
        }
    }

    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128
        )
    }

    #[must_use]
    pub fn from_min_max(min: i128, max: i128) -> Self {
        if min < 0 {
            if min >= i128::from(i8::MIN) && max <= i128::from(i8::MAX) {
                IntReprType::I8
            } else if min >= i128::from(i16::MIN) && max <= i128::from(i16::MAX) {
                IntReprType::I16
            } else if min >= i128::from(i32::MIN) && max <= i128::from(i32::MAX) {
                IntReprType::I32
            } else if min >= i128::from(i64::MIN) && max <= i128::from(i64::MAX) {
                IntReprType::I64
            } else {
                IntReprType::I128
            }
        } else if max <= i128::from(u8::MAX) {
            IntReprType::U8
        } else if max <= i128::from(u16::MAX) {
            IntReprType::U16
        } else if max <= i128::from(u32::MAX) {
            IntReprType::U32
        } else if max <= i128::from(u64::MAX) {
            IntReprType::U64
        } else {
            IntReprType::U128
        }
    }

    #[must_use]
    pub fn unsigned(self) -> Self {
        match self {
            Self::U8 | Self::I8 => Self::U8,
            Self::U16 | Self::I16 => Self::U16,
            Self::U32 | Self::I32 => Self::U32,
            Self::U64 | Self::I64 => Self::U64,
            Self::U128 | Self::I128 => Self::U128,
        }
    }

    #[must_use]
    pub fn signed(self) -> Self {
        match self {
            Self::U8 | Self::I8 => Self::I8,
            Self::U16 | Self::I16 => Self::I16,
            Self::U32 | Self::I32 => Self::I32,
            Self::U64 | Self::I64 => Self::I64,
            Self::U128 | Self::I128 => Self::I128,
        }
    }

    #[must_use]
    pub fn bits(self) -> u32 {
        match self {
            Self::U8 | Self::I8 => 8,
            Self::U16 | Self::I16 => 16,
            Self::U32 | Self::I32 => 32,
            Self::U64 | Self::I64 => 64,
            Self::U128 | Self::I128 => 128,
        }
    }
}

impl RustType for IntReprType {
    fn as_rust_type(&self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
        }
    }
}

impl CppType for IntReprType {
    fn as_cpp_type(&self) -> &'static str {
        match self {
            Self::U8 => "uint8_t",
            Self::U16 => "uint16_t",
            Self::U32 => "uint32_t",
            Self::U64 => "uint64_t",
            Self::U128 => "unsigned __int128",
            Self::I8 => "int8_t",
            Self::I16 => "int16_t",
            Self::I32 => "int32_t",
            Self::I64 => "int64_t",
            Self::I128 => "__int128_t",
        }
    }
}

impl RustIntegerLiteral for IntReprType {
    fn literal(&self, value: i64) -> Literal {
        match self {
            Self::U8 => Literal::u8_suffixed(u8::try_from(value).unwrap()),
            Self::U16 => Literal::u16_suffixed(u16::try_from(value).unwrap()),
            Self::U32 => Literal::u32_suffixed(u32::try_from(value).unwrap()),
            Self::U64 => Literal::u64_suffixed(u64::try_from(value).unwrap()),
            Self::U128 => Literal::u128_suffixed(u128::try_from(value).unwrap()),
            Self::I8 => Literal::i8_suffixed(i8::try_from(value).unwrap()),
            Self::I16 => Literal::i16_suffixed(i16::try_from(value).unwrap()),
            Self::I32 => Literal::i32_suffixed(i32::try_from(value).unwrap()),
            Self::I64 => Literal::i64_suffixed(value),
            Self::I128 => Literal::i128_suffixed(i128::from(value)),
        }
    }
}
