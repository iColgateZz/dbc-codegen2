use crate::ir::{ValueDescription, map_into};
use can_dbc::ValDescription as ParsedValueDescription;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SignalValueEnumIdx(pub usize);

#[derive(Debug, Clone)]
pub struct SignalValueEnum {
    pub name: String,
    pub variants: Vec<ValueDescription>,
}

impl SignalValueEnum {
    pub fn from_parsed(name: String, variants: Vec<ParsedValueDescription>) -> Self {
        Self {
            name,
            variants: map_into(variants),
        }
    }
}