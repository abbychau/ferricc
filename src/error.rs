use thiserror::Error;

use crate::ast::Location;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Lexical error at {location}: {message}")]
    LexicalError { location: Location, message: String },

    #[error("Syntax error at {location}: {message}")]
    SyntaxError { location: Location, message: String },

    #[error("Type error at {location}: {message}")]
    TypeError { location: Location, message: String },

    #[error("Semantic error at {location}: {message}")]
    SemanticError { location: Location, message: String },

    #[error("Code generation error: {message}")]
    CodeGenError { message: String },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Preprocessor error at {location}: {message}")]
    PreprocessorError { location: Location, message: String },
}

pub type Result<T> = std::result::Result<T, CompilerError>;

/// Helper function to create a lexical error
pub fn lexical_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::LexicalError {
        location: location.clone(),
        message: message.into(),
    }
}

/// Helper function to create a syntax error
pub fn syntax_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::SyntaxError {
        location: location.clone(),
        message: message.into(),
    }
}

/// Helper function to create a type error
pub fn type_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::TypeError {
        location: location.clone(),
        message: message.into(),
    }
}

/// Helper function to create a semantic error
pub fn semantic_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::SemanticError {
        location: location.clone(),
        message: message.into(),
    }
}

/// Helper function to create a code generation error
pub fn codegen_error(message: impl Into<String>) -> CompilerError {
    CompilerError::CodeGenError {
        message: message.into(),
    }
}

/// Helper function to create a preprocessor error
pub fn preprocessor_error(location: &Location, message: impl Into<String>) -> CompilerError {
    CompilerError::PreprocessorError {
        location: location.clone(),
        message: message.into(),
    }
}
