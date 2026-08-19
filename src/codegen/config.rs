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
    Getter,
    Setter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CppCodeInjectionPoint {
    Header,
    Footer,
    ErrorEnum,
    MessageVariant,
    SignalValueEnum,
    MuxVariant,
    MuxVariantClass,
    MuxVariantClassPublic,
    MuxVariantClassPrivate,
    MessageClass,
    MessageClassPublic,
    MessageClassPrivate,
}

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub inputs: Vec<String>,
    pub output: String,
    pub lang: Language,
    pub enum_other: bool,
    pub enum_dedup: bool,
    pub allow_unrestricted_ranges: bool,
    pub rust_code_injections: HashMap<RustCodeInjectionPoint, Vec<String>>,
    pub cpp_code_injections: HashMap<CppCodeInjectionPoint, Vec<String>>,
    pub generate_tests: bool,
    pub separate: bool,
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

    pub fn add_cpp_code_injection(
        &mut self,
        point: CppCodeInjectionPoint,
        code: impl Into<String>,
    ) {
        self.cpp_code_injections
            .entry(point)
            .or_default()
            .push(code.into());
    }
}
