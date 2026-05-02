use heck::ToSnakeCase;
use heck::ToUpperCamelCase;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub prefix: String,
    pub raw: String,
    pub postfix: String,
}

impl Identifier {
    pub fn from_raw(raw: String) -> Self {
        Self {
            prefix: String::new(),
            raw,
            postfix: String::new(),
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn rendered(&self) -> String {
        format!("{}{}{}", self.prefix, self.raw, self.postfix)
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

    pub fn ensure_name_validity(&mut self) {
        if !is_valid_identifier(&self.rendered()) {
            self.prefix = format!("x{}", self.prefix);
        }
    }

    pub fn upper_camel_with_numeric_postfix(&self) -> String {
        let numeric_postfix: String = self
            .postfix
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect();

        format!("{}{}{}", self.prefix, self.raw, numeric_postfix).to_upper_camel_case()
    }
}

pub fn is_valid_identifier(candidate: &str) -> bool {
    let mut chars = candidate.chars();

    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        return false;
    }

    !is_rust_keyword(candidate)
}

fn is_rust_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s.to_lowercase().as_str())
}

const KEYWORDS: [&str; 53] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "union",
    // Internal names
    "_other",
];
