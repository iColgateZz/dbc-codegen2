use can_dbc::SignalExtendedValueType as ParsedExtendedValueType;

#[derive(Debug, Clone, Copy)]
pub enum ExtendedValueType {
    Integer,
    Float32,
    Double64,
}

impl From<ParsedExtendedValueType> for ExtendedValueType {
    fn from(value: ParsedExtendedValueType) -> Self {
        match value {
            ParsedExtendedValueType::SignedOrUnsignedInteger => Self::Integer,
            ParsedExtendedValueType::IEEEfloat32Bit => Self::Float32,
            ParsedExtendedValueType::IEEEdouble64bit => Self::Double64,
        }
    }
}
