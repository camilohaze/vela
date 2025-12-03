/*!
# Compiler Error Handling

Comprehensive error handling system for the Vela compiler with detailed
diagnostics, source locations, and severity levels.
*/

use std::fmt;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Result type for compilation operations
pub type CompileResult<T> = Result<T, CompileError>;

/// Compilation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// I/O error
    Io {
        path: PathBuf,
        error: String, // Store error message instead of Error type
    },
    /// Lexical error
    Lexical(LexicalError),
    /// Parse error
    Parse(ParseError),
    /// Semantic error
    Semantic(SemanticError),
    /// Code generation error
    Codegen(CodegenError),
    /// Internal compiler error
    Internal(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Io { path, error } => {
                write!(f, "I/O error in {}: {}", path.display(), error)
            }
            CompileError::Lexical(err) => write!(f, "Lexical error: {}", err),
            CompileError::Parse(err) => write!(f, "Parse error: {}", err),
            CompileError::Semantic(err) => write!(f, "Semantic error: {}", err),
            CompileError::Codegen(err) => write!(f, "Code generation error: {}", err),
            CompileError::Internal(msg) => write!(f, "Internal compiler error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Lexical analysis errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalError {
    pub message: String,
    pub location: SourceLocation,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

/// Parse errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub location: SourceLocation,
    pub expected: Vec<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

/// Semantic analysis errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
    pub location: SourceLocation,
    pub error_code: Option<String>,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

/// Code generation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub message: String,
    pub location: Option<SourceLocation>,
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(loc) => write!(f, "{} at {}", self.message, loc),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Source location in a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from start of file
    pub offset: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Create an unknown location
    pub fn unknown() -> Self {
        Self { line: 0, column: 0, offset: 0 }
    }

    /// Check if this is an unknown location
    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0 && self.offset == 0
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unknown() {
            write!(f, "<unknown>")
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

/// A diagnostic message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
    pub code: Option<String>,
    pub hints: Vec<String>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(severity: Severity, message: String, location: SourceLocation) -> Self {
        Self {
            severity,
            message,
            location,
            code: None,
            hints: Vec::new(),
        }
    }

    /// Add an error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add hints
    pub fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints = hints;
        self
    }
}

/// Collection of diagnostics
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Create a new diagnostics collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error
    pub fn error(&mut self, message: impl Into<String>, location: SourceLocation) {
        self.add(Diagnostic::new(Severity::Error, message.into(), location));
    }

    /// Add a warning
    pub fn warning(&mut self, message: impl Into<String>, location: SourceLocation) {
        self.add(Diagnostic::new(Severity::Warning, message.into(), location));
    }

    /// Add an info message
    pub fn info(&mut self, message: impl Into<String>, location: SourceLocation) {
        self.add(Diagnostic::new(Severity::Info, message.into(), location));
    }

    /// Get all diagnostics
    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get errors only
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error)
    }

    /// Get warnings only
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning)
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    /// Get the number of diagnostics
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Extend diagnostics from lexer errors
    pub fn extend_from_lexer(&mut self, lexer_errors: &[crate::lexer::LexError]) {
        for error in lexer_errors {
            let location = match error {
                crate::lexer::LexError::UnexpectedCharacter(_, pos) |
                crate::lexer::LexError::UnterminatedString(pos) |
                crate::lexer::LexError::InvalidEscapeSequence(_, pos) |
                crate::lexer::LexError::InvalidNumberLiteral(_, pos) => {
                    SourceLocation::new(pos.line, pos.column, 0) // offset not tracked in lexer
                }
            };

            let message = match error {
                crate::lexer::LexError::UnexpectedCharacter(c, _) =>
                    format!("Unexpected character: '{}'", c),
                crate::lexer::LexError::UnterminatedString(_) =>
                    "Unterminated string literal".to_string(),
                crate::lexer::LexError::InvalidEscapeSequence(seq, _) =>
                    format!("Invalid escape sequence: {}", seq),
                crate::lexer::LexError::InvalidNumberLiteral(num, _) =>
                    format!("Invalid number literal: {}", num),
            };

            self.add(Diagnostic::new(Severity::Error, message, location));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new(10, 5, 42);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.offset, 42);
        assert!(!loc.is_unknown());

        let unknown = SourceLocation::unknown();
        assert!(unknown.is_unknown());
    }

    #[test]
    fn test_diagnostics() {
        let mut diagnostics = Diagnostics::new();

        let loc = SourceLocation::new(1, 1, 0);
        diagnostics.error("Test error", loc);
        diagnostics.warning("Test warning", loc);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.has_errors());
        assert_eq!(diagnostics.errors().count(), 1);
        assert_eq!(diagnostics.warnings().count(), 1);
    }

    #[test]
    fn test_diagnostic_with_code_and_hints() {
        let loc = SourceLocation::new(1, 1, 0);
        let diagnostic = Diagnostic::new(Severity::Error, "Test".to_string(), loc)
            .with_code("E001")
            .with_hints(vec!["Try this".to_string(), "Or this".to_string()]);

        assert_eq!(diagnostic.code.as_deref(), Some("E001"));
        assert_eq!(diagnostic.hints.len(), 2);
    }
}