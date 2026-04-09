use crate::ir::DbcFile;

pub trait CheckNode {
    fn check(&self, file: &DbcFile, diagnostics: &mut Diagnostics);
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
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn error(&mut self, msg: impl Into<String>) {
        self.push(DiagnosticLevel::Error, msg);
    }

    pub fn warning(&mut self, msg: impl Into<String>) {
        self.push(DiagnosticLevel::Warning, msg);
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.push(DiagnosticLevel::Info, msg);
    }

    fn push(&mut self, level: DiagnosticLevel, msg: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            level,
            message: msg.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error))
    }

    pub fn emit(&self) {
        for diag in &self.diagnostics {
            match diag.level {
                DiagnosticLevel::Error => eprintln!("error: {}", diag.message),
                DiagnosticLevel::Warning => eprintln!("warning: {}", diag.message),
                DiagnosticLevel::Info => eprintln!("info: {}", diag.message),
            }
        }
    }
}
