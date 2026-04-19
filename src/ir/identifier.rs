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

    pub fn rendered(&self) -> String {
        format!("{}{}", self.raw, self.postfix)
    }

    pub fn lower(&self) -> String {
        self.rendered().to_lowercase()
    }

    pub fn upper_camel(&self) -> String {
        self.rendered().to_upper_camel_case()
    }

    pub fn snake_case(&self) -> String {
        self.rendered().to_snake_case()
    }
}
