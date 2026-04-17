use heck::ToUpperCamelCase;
use heck::ToSnakeCase;

#[derive(Debug, Clone)]
pub struct Identifier{
    pub raw: String,
    pub postfix: String,
}

impl Identifier {
    pub fn from_raw(raw: String) -> Self {
        Self {
            raw,
            postfix: String::new(),
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn lower(&self) -> String {
        format!("{}{}", self.raw, self.postfix).to_lowercase()
    }

    pub fn upper_camel(&self) -> String {
        format!("{}{}", self.raw, self.postfix).to_upper_camel_case()
    }

    pub fn snake_case(&self) -> String {
        format!("{}{}", self.raw, self.postfix).to_snake_case()
    }
}
