pub struct Generator {
    buffer: String,
    indent_level: usize,
    indent_str: String,
}

impl Generator {
    pub fn new() -> Self {
        Self::with_indent("  ")
    }

    pub fn with_indent(indent: &str) -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            indent_str: indent.to_string(),
        }
    }

    pub fn line(&mut self, text: &str) {
        self.push_indent();
        self.buffer.push_str(text);
        self.buffer.push('\n');
    }

    pub fn start_block(&mut self, text: &str) {
        self.push_indent();
        self.buffer.push_str(text);
        self.buffer.push_str(" {\n");
        self.indent_level += 1;
    }

    pub fn end_block(&mut self, text: &str) {
        if !text.is_empty() {
            self.line(text);
        }
        self.indent_level = self.indent_level.saturating_sub(1);
        self.push_indent();
        self.buffer.push_str("};\n");
    }

    pub fn get(&self) -> &str {
        &self.buffer
    }

    pub fn into_string(self) -> String {
        self.buffer
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.buffer.push_str(&self.indent_str);
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! line {
    ($gen:expr, $fmt:literal $(, $args:expr)* $(,)?) => {
        $gen.line(&format!($fmt $(, $args)*))
    };
}

#[macro_export]
macro_rules! start_block {
    ($gen:expr, $fmt:literal $(, $args:expr)* $(,)?) => {
        $gen.start_block(&format!($fmt $(, $args)*))
    };
}

#[macro_export]
macro_rules! end_block {
    ($gen:expr, $fmt:literal $(, $args:expr)* $(,)?) => {
        $gen.end_block(&format!($fmt $(, $args)*))
    };
}

#[macro_export]
macro_rules! empty {
    ($gen:expr) => {
        $gen.line("")
    };
}