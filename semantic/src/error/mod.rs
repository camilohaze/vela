use thiserror::Error;
use crate::symbol::Span;

/// Semantic error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SemanticError {
    #[error("Undefined variable '{name}' at {span:?}")]
    UndefinedVariable {
        name: String,
        span: Span,
    },

    #[error("Variable '{name}' is already defined (original: {original:?}, duplicate: {duplicate:?})")]
    AlreadyDefined {
        name: String,
        original: Span,
        duplicate: Span,
    },

    #[error("Variable '{name}' is not in scope at {span:?}")]
    NotInScope {
        name: String,
        span: Span,
    },

    #[error("Cannot reassign immutable variable '{name}' at {span:?}")]
    CannotReassignImmutable {
        name: String,
        span: Span,
    },

    #[error("Invalid shadowing of '{name}' (outer: {outer:?}, inner: {inner:?})")]
    InvalidShadowing {
        name: String,
        outer: Span,
        inner: Span,
    },

    #[error("Use of variable '{name}' before definition (use: {use_span:?}, def: {def_span:?})")]
    UseBeforeDefinition {
        name: String,
        use_span: Span,
        def_span: Span,
    },

    #[error("Cannot capture variable '{name}' in closure at {span:?}")]
    CannotCaptureVariable {
        name: String,
        span: Span,
    },

    #[error("Function '{name}' is already defined (original: {original:?}, duplicate: {duplicate:?})")]
    FunctionAlreadyDefined {
        name: String,
        original: Span,
        duplicate: Span,
    },

    #[error("Undefined function '{name}' at {span:?}")]
    UndefinedFunction {
        name: String,
        span: Span,
    },

    #[error("Class '{name}' is already defined (original: {original:?}, duplicate: {duplicate:?})")]
    ClassAlreadyDefined {
        name: String,
        original: Span,
        duplicate: Span,
    },

    #[error("Undefined class '{name}' at {span:?}")]
    UndefinedClass {
        name: String,
        span: Span,
    },
}

impl SemanticError {
    /// Get the span where the error occurred
    pub fn span(&self) -> Span {
        match self {
            SemanticError::UndefinedVariable { span, .. } => *span,
            SemanticError::AlreadyDefined { duplicate, .. } => *duplicate,
            SemanticError::NotInScope { span, .. } => *span,
            SemanticError::CannotReassignImmutable { span, .. } => *span,
            SemanticError::InvalidShadowing { inner, .. } => *inner,
            SemanticError::UseBeforeDefinition { use_span, .. } => *use_span,
            SemanticError::CannotCaptureVariable { span, .. } => *span,
            SemanticError::FunctionAlreadyDefined { duplicate, .. } => *duplicate,
            SemanticError::UndefinedFunction { span, .. } => *span,
            SemanticError::ClassAlreadyDefined { duplicate, .. } => *duplicate,
            SemanticError::UndefinedClass { span, .. } => *span,
        }
    }

    /// Check if this is a definition error
    pub fn is_definition_error(&self) -> bool {
        matches!(
            self,
            SemanticError::AlreadyDefined { .. }
                | SemanticError::FunctionAlreadyDefined { .. }
                | SemanticError::ClassAlreadyDefined { .. }
        )
    }

    /// Check if this is a usage error
    pub fn is_usage_error(&self) -> bool {
        matches!(
            self,
            SemanticError::UndefinedVariable { .. }
                | SemanticError::UndefinedFunction { .. }
                | SemanticError::UndefinedClass { .. }
                | SemanticError::NotInScope { .. }
                | SemanticError::UseBeforeDefinition { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined_variable_error() {
        let error = SemanticError::UndefinedVariable {
            name: "x".to_string(),
            span: Span::new(0, 1),
        };

        assert_eq!(error.span(), Span::new(0, 1));
        assert!(error.is_usage_error());
        assert!(!error.is_definition_error());
    }

    #[test]
    fn test_already_defined_error() {
        let error = SemanticError::AlreadyDefined {
            name: "x".to_string(),
            original: Span::new(0, 1),
            duplicate: Span::new(10, 11),
        };

        assert_eq!(error.span(), Span::new(10, 11));
        assert!(error.is_definition_error());
        assert!(!error.is_usage_error());
    }

    #[test]
    fn test_cannot_reassign_immutable_error() {
        let error = SemanticError::CannotReassignImmutable {
            name: "PI".to_string(),
            span: Span::new(20, 22),
        };

        assert_eq!(error.span(), Span::new(20, 22));
        assert!(!error.is_usage_error());
        assert!(!error.is_definition_error());
    }

    #[test]
    fn test_invalid_shadowing_error() {
        let error = SemanticError::InvalidShadowing {
            name: "x".to_string(),
            outer: Span::new(0, 1),
            inner: Span::new(10, 11),
        };

        assert_eq!(error.span(), Span::new(10, 11));
    }

    #[test]
    fn test_use_before_definition_error() {
        let error = SemanticError::UseBeforeDefinition {
            name: "x".to_string(),
            use_span: Span::new(5, 6),
            def_span: Span::new(10, 11),
        };

        assert_eq!(error.span(), Span::new(5, 6));
        assert!(error.is_usage_error());
    }

    #[test]
    fn test_cannot_capture_variable_error() {
        let error = SemanticError::CannotCaptureVariable {
            name: "x".to_string(),
            span: Span::new(15, 16),
        };

        assert_eq!(error.span(), Span::new(15, 16));
    }

    #[test]
    fn test_function_already_defined_error() {
        let error = SemanticError::FunctionAlreadyDefined {
            name: "add".to_string(),
            original: Span::new(0, 3),
            duplicate: Span::new(20, 23),
        };

        assert!(error.is_definition_error());
        assert!(!error.is_usage_error());
    }

    #[test]
    fn test_undefined_function_error() {
        let error = SemanticError::UndefinedFunction {
            name: "compute".to_string(),
            span: Span::new(30, 37),
        };

        assert!(error.is_usage_error());
        assert!(!error.is_definition_error());
    }

    #[test]
    fn test_class_errors() {
        let error1 = SemanticError::ClassAlreadyDefined {
            name: "Person".to_string(),
            original: Span::new(0, 6),
            duplicate: Span::new(50, 56),
        };

        let error2 = SemanticError::UndefinedClass {
            name: "Animal".to_string(),
            span: Span::new(100, 106),
        };

        assert!(error1.is_definition_error());
        assert!(error2.is_usage_error());
    }
}
