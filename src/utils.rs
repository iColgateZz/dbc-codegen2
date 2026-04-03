pub trait ToUpperCamelCase {
    fn to_upper_camelcase(&self) -> String;
}

impl ToUpperCamelCase for str {
    fn to_upper_camelcase(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut capitalize_next = true;

        for c in self.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        }

        result
    }
}

#[derive(clap::ValueEnum, Clone)]
pub enum Language {
    Rust,
    Cpp,
}
impl Language {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Cpp  => "cpp",
        }
    }
}
