use crate::utils::Language;

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub input: String,
    pub output: String,
    pub lang: Language,
    pub no_enum_other: bool,
}
