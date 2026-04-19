use std::collections::HashMap;

use crate::utils::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RustCodeInjectionPoint {
    MessageStruct,
    MessageEnum,
    SignalValueEnum,
    MuxEnum,
    MuxVariantStruct,
    ErrorEnum,
}

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub inputs: Vec<String>,
    pub output: String,
    pub lang: Language,
    pub no_enum_other: bool,
    pub no_enum_dedup: bool,
    pub zero_zero_range_allows_all: bool,
    pub rust_code_injections: HashMap<RustCodeInjectionPoint, Vec<String>>,
}

impl CodegenConfig {
    pub fn add_rust_code_injection(
        &mut self,
        point: RustCodeInjectionPoint,
        code: impl Into<String>,
    ) {
        self.rust_code_injections
            .entry(point)
            .or_default()
            .push(code.into());
    }
}