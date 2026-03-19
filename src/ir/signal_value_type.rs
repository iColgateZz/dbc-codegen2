use proc_macro2::Literal;

pub trait RustType {
    fn as_rust_type(&self) -> &'static str;
}

pub trait RustLiteral {
    fn literal(&self, value: i64) -> Literal;
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

#[derive(Debug, Clone, PartialEq)]
pub enum EnumCoverage {
    Exhaustive,
    Partial,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalType {
    Float32,
    Float64,
    Integer(IntReprType),
    Enum {
        coverage: EnumCoverage, 
        repr: IntReprType
    },
}

impl RustType for PhysicalType {
    fn as_rust_type(&self) -> &'static str {
        match self {
            PhysicalType::Float32 => "f32",
            PhysicalType::Float64 => "f64",
            PhysicalType::Integer(v) => v.as_rust_type(),
            PhysicalType::Enum { repr, .. } => repr.as_rust_type(),
        }
    }
}

impl RustLiteral for PhysicalType {
    fn literal(&self, value: i64) -> Literal {
        match self {
            PhysicalType::Float32 => Literal::f32_suffixed(value as f32),
            PhysicalType::Float64 => Literal::f64_suffixed(value as f64),

            PhysicalType::Integer(repr) => repr.literal(value),

            PhysicalType::Enum { repr, .. } => repr.literal(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntReprType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    #[default]
    I64,
}

impl IntReprType {
    pub fn from_size_sign(size: u64, signed: bool) -> IntReprType {
        match (signed, size) {
            (false, 0..=8) => IntReprType::U8,
            (false, 9..=16) => IntReprType::U16,
            (false, 17..=32) => IntReprType::U32,
            (false, _) => IntReprType::U64,

            (true, 0..=8) => IntReprType::I8,
            (true, 9..=16) => IntReprType::I16,
            (true, 17..=32) => IntReprType::I32,
            (true, _) => IntReprType::I64,
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
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
        }
    }
}

impl RustLiteral for IntReprType {
    fn literal(&self, value: i64) -> Literal {
        match self {
            Self::U8  => Literal::u8_suffixed(value as u8),
            Self::U16 => Literal::u16_suffixed(value as u16),
            Self::U32 => Literal::u32_suffixed(value as u32),
            Self::U64 => Literal::u64_suffixed(value as u64),
            Self::I8  => Literal::i8_suffixed(value as i8),
            Self::I16 => Literal::i16_suffixed(value as i16),
            Self::I32 => Literal::i32_suffixed(value as i32),
            Self::I64 => Literal::i64_suffixed(value),
        }
    }
}