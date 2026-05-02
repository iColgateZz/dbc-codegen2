#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Language {
    Rust,
    Cpp,
}
impl Language {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Cpp => "hpp",
        }
    }
}
