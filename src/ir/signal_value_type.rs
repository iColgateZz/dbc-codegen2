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
        repr: IntReprType,
    },
}

impl PhysicalType {
    pub fn is_float(&self) -> bool {
        match self {
            PhysicalType::Float32 | PhysicalType::Float64 => true,
            _ => false,
        }
    }
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

impl CppType for PhysicalType {
    fn as_cpp_type(&self) -> &'static str {
        match self {
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
            PhysicalType::Integer(repr) => repr.literal(value),
            PhysicalType::Enum { repr, .. } => repr.literal(value),
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

impl CppType for IntReprType {
    fn as_cpp_type(&self) -> &'static str {
        match self {
            Self::U8 => "uint8_t",
            Self::U16 => "uint16_t",
            Self::U32 => "uint32_t",
            Self::U64 => "uint64_t",
            Self::I8 => "int8_t",
            Self::I16 => "int16_t",
            Self::I32 => "int32_t",
            Self::I64 => "int64_t",
        }
    }
}

impl RustIntegerLiteral for IntReprType {
    fn literal(&self, value: i64) -> Literal {
        match self {
            Self::U8 => Literal::u8_suffixed(value as u8),
            Self::U16 => Literal::u16_suffixed(value as u16),
            Self::U32 => Literal::u32_suffixed(value as u32),
            Self::U64 => Literal::u64_suffixed(value as u64),
            Self::I8 => Literal::i8_suffixed(value as i8),
            Self::I16 => Literal::i16_suffixed(value as i16),
            Self::I32 => Literal::i32_suffixed(value as i32),
            Self::I64 => Literal::i64_suffixed(value),
        }
    }
}
