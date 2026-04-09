use crate::ir::DbcFile;

pub trait CheckNode {
    fn check(&self, file: &DbcFile, diagnostics: &Diagnostics);
}

#[derive(Debug)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub infos: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn error(&mut self, msg: String) {
        self.errors.push(Diagnostic { level: DiagnosticLevel::Error, message: msg });
    }

    pub fn warning(&mut self, msg: String) {
        self.warnings.push(Diagnostic { level: DiagnosticLevel::Warning, message: msg });
    }

    pub fn info(&mut self, msg: String) {
        self.infos.push(Diagnostic { level: DiagnosticLevel::Info, message: msg });
    }
}
