use can_dbc::ValDescription as ParsedValDescription;

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
