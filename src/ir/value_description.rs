use can_dbc::ValDescription as ParsedValDescription;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ValueDescriptionIdx(pub usize);

#[derive(Debug, Clone)]
pub struct ValueDescription {
    pub value: i64,
    pub description: String,
}

impl From<ParsedValDescription> for ValueDescription {
    fn from(value: ParsedValDescription) -> Self {
        Self {
            value: value.id,
            description: value.description,
        }
    }
}
