pub trait ToUpperCamelCase {
    fn to_upper_camelcase(&self) -> String;
}

impl ToUpperCamelCase for str {
    fn to_upper_camelcase(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut capitalize_next = true;

        for c in self.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        }

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReprType {
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

impl ReprType {
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
}

pub fn infer_repr_type(iter: impl IntoIterator<Item = i64>) -> ReprType {
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
        return ReprType::U8;
    }

    if min >= 0 {
        match max as u64 {
            0..=255 => ReprType::U8,
            256..=65535 => ReprType::U16,
            v if v <= u32::MAX as u64 => ReprType::U32,
            _ => ReprType::U64,
        }
    } else {
        match (min, max) {
            (mn, mx) if mn >= i8::MIN as i64 && mx <= i8::MAX as i64 => ReprType::I8,
            (mn, mx) if mn >= i16::MIN as i64 && mx <= i16::MAX as i64 => ReprType::I16,
            (mn, mx) if mn >= i32::MIN as i64 && mx <= i32::MAX as i64 => ReprType::I32,
            _ => ReprType::I64,
        }
    }
}
