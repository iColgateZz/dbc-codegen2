use proc_macro2::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum RawType {
    Float32,
    Float64,
    Integer(IntReprType),
}

impl RawType {
    pub fn as_rust_type(&self) -> &'static str {
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

impl PhysicalType {
    pub fn as_rust_type(&self) -> &'static str {
        match self {
            PhysicalType::Float32 => "f32",
            PhysicalType::Float64 => "f64",
            PhysicalType::Integer(v) => v.as_rust_type(),
            PhysicalType::Enum { coverage: _, repr } => repr.as_rust_type(),
        }
    }

    pub fn literal(&self, value: i64) -> Literal {
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
    pub fn as_rust_type(&self) -> &'static str {
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
    
    pub fn literal(&self, value: i64) -> proc_macro2::Literal {
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

    pub fn from_iter(iter: impl IntoIterator<Item = i64>) -> IntReprType {
        let mut min = i64::MAX;
        let mut max = i64::MIN;
        let mut any = false;
    
        for v in iter {
            any = true;
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
    
        if !any {
            return IntReprType::U8;
        }
    
        if min >= 0 {
            match max as u64 {
                0..=255 => IntReprType::U8,
                256..=65535 => IntReprType::U16,
                v if v <= u32::MAX as u64 => IntReprType::U32,
                _ => IntReprType::U64,
            }
        } else {
            match (min, max) {
                (mn, mx) if mn >= i8::MIN as i64 && mx <= i8::MAX as i64 => IntReprType::I8,
                (mn, mx) if mn >= i16::MIN as i64 && mx <= i16::MAX as i64 => IntReprType::I16,
                (mn, mx) if mn >= i32::MIN as i64 && mx <= i32::MAX as i64 => IntReprType::I32,
                _ => IntReprType::I64,
            }
        }
    }

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
