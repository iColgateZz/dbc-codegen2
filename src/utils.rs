#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Language {
    Rust,
    Cpp,
}
impl Language {
    #[must_use]
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Cpp => "hpp",
        }
    }
}
