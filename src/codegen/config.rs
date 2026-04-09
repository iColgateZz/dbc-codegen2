use crate::utils::Language;

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub input: String,
    pub output: String,
    pub lang: Language,
    pub no_enum_other: bool,
    pub no_enum_dedup: bool,
    pub zero_zero_range_allows_all: bool,
}
