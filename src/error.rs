use crate::span::Span;
use std::fmt;

#[derive(Debug, Clone)]
pub enum CorvoError {
    Lexing { message: String, span: Option<Span> },
    Parsing { message: String, span: Option<Span> },
    Type { message: String, span: Option<Span> },
    Runtime { message: String, span: Option<Span> },
    Assertion { message: String, span: Option<Span> },
    FileSystem { message: String, span: Option<Span> },
    Network { message: String, span: Option<Span> },
    UnknownFunction { name: String, span: Option<Span> },
    StaticModification { name: String, span: Option<Span> },
    DivisionByZero { span: Option<Span> },
    VariableNotFound { name: String, span: Option<Span> },
    StaticNotFound { name: String, span: Option<Span> },
    Io { message: String, span: Option<Span> },
    InvalidArgument { message: String, span: Option<Span> },
}

impl fmt::Display for CorvoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexing { message, .. } => write!(f, "Lexing error: {}", message),
            Self::Parsing { message, .. } => write!(f, "Parsing error: {}", message),
            Self::Type { message, .. } => write!(f, "Type error: {}", message),
            Self::Runtime { message, .. } => write!(f, "Runtime error: {}", message),
            Self::Assertion { message, .. } => write!(f, "Assertion failed: {}", message),
            Self::FileSystem { message, .. } => write!(f, "File system error: {}", message),
            Self::Network { message, .. } => write!(f, "Network error: {}", message),
            Self::UnknownFunction { name, .. } => write!(f, "Unknown function: {}", name),
            Self::StaticModification { name, .. } => {
                write!(f, "Static modification error: {} cannot be modified", name)
            }
            Self::DivisionByZero { .. } => write!(f, "Division by zero"),
            Self::VariableNotFound { name, .. } => write!(f, "Variable not found: {}", name),
            Self::StaticNotFound { name, .. } => write!(f, "Static not found: {}", name),
            Self::Io { message, .. } => write!(f, "IO error: {}", message),
            Self::InvalidArgument { message, .. } => write!(f, "Invalid argument: {}", message),
        }
    }
}

impl std::error::Error for CorvoError {}

impl CorvoError {
    pub fn lexing(message: impl Into<String>) -> Self {
        Self::Lexing {
            message: message.into(),
            span: None,
        }
    }

    pub fn parsing(message: impl Into<String>) -> Self {
        Self::Parsing {
            message: message.into(),
            span: None,
        }
    }

    pub fn r#type(message: impl Into<String>) -> Self {
        Self::Type {
            message: message.into(),
            span: None,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            span: None,
        }
    }

    pub fn assertion(message: impl Into<String>) -> Self {
        Self::Assertion {
            message: message.into(),
            span: None,
        }
    }

    pub fn unknown_function(name: impl Into<String>) -> Self {
        Self::UnknownFunction {
            name: name.into(),
            span: None,
        }
    }

    pub fn variable_not_found(name: impl Into<String>) -> Self {
        Self::VariableNotFound {
            name: name.into(),
            span: None,
        }
    }

    pub fn static_not_found(name: impl Into<String>) -> Self {
        Self::StaticNotFound {
            name: name.into(),
            span: None,
        }
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            message: message.into(),
            span: None,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            span: None,
        }
    }

    pub fn division_by_zero() -> Self {
        Self::DivisionByZero { span: None }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            span: None,
        }
    }

    pub fn file_system(message: impl Into<String>) -> Self {
        Self::FileSystem {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        match &mut self {
            Self::Lexing { span: s, .. }
            | Self::Parsing { span: s, .. }
            | Self::Type { span: s, .. }
            | Self::Runtime { span: s, .. }
            | Self::Assertion { span: s, .. }
            | Self::FileSystem { span: s, .. }
            | Self::Network { span: s, .. }
            | Self::UnknownFunction { span: s, .. }
            | Self::StaticModification { span: s, .. }
            | Self::DivisionByZero { span: s }
            | Self::VariableNotFound { span: s, .. }
            | Self::StaticNotFound { span: s, .. }
            | Self::Io { span: s, .. }
            | Self::InvalidArgument { span: s, .. } => *s = Some(span),
        }
        self
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Lexing { .. } => 1,
            Self::Parsing { .. } => 2,
            Self::Type { .. } => 3,
            Self::Runtime { .. } => 4,
            Self::Assertion { .. } => 5,
            Self::FileSystem { .. } => 6,
            Self::Network { .. } => 7,
            Self::UnknownFunction { .. } => 8,
            Self::StaticModification { .. } => 9,
            Self::DivisionByZero { .. } => 10,
            Self::VariableNotFound { .. } => 11,
            Self::StaticNotFound { .. } => 12,
            Self::Io { .. } => 13,
            Self::InvalidArgument { .. } => 14,
        }
    }
}

pub type CorvoResult<T> = Result<T, CorvoError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Position;

    #[test]
    fn test_error_with_span() {
        let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
        let error = CorvoError::lexing("unexpected token").with_span(span);
        match error {
            CorvoError::Lexing {
                message,
                span: Some(s),
            } => {
                assert_eq!(message, "unexpected token");
                assert_eq!(s.start.line, 1);
            }
            _ => panic!("Expected Lexing error with span"),
        }
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(CorvoError::lexing("test").exit_code(), 1);
        assert_eq!(CorvoError::parsing("test").exit_code(), 2);
        assert_eq!(CorvoError::runtime("test").exit_code(), 4);
    }
}
