use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{preprocessor_error, Result};
use crate::lexer::{Lexer, Token, TokenKind};

/// Preprocessor for C source code
pub struct Preprocessor {
    include_paths: Vec<PathBuf>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            include_paths: vec![],
        }
    }

    /// Add an include path
    pub fn add_include_path(&mut self, path: impl AsRef<Path>) {
        self.include_paths.push(path.as_ref().to_path_buf());
    }

    /// Define a macro (stub for now)
    pub fn define_macro(&mut self, _name: &str, _value: Vec<Token>) {
        // Not implemented yet
    }

    /// Preprocess a token stream
    pub fn preprocess(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let token = &tokens[i];

            if token.kind == TokenKind::Hash {
                // Preprocessor directive
                i += 1;

                if i >= tokens.len() {
                    return Err(preprocessor_error(
                        &token.location,
                        "Unexpected end of file after #",
                    ));
                }

                let directive = &tokens[i];

                match &directive.kind {
                    TokenKind::Identifier(name) => {
                        match name.as_str() {
                            "include" => {
                                i = self.process_include(&tokens, i, &mut result)?;
                            }
                            _ => {
                                // Skip to the next token
                                i += 1;
                            }
                        }
                    }
                    _ => {
                        // Skip to the next token
                        i += 1;
                    }
                }
            } else {
                result.push(token.clone());
                i += 1;
            }
        }

        Ok(result)
    }

    /// Process #include directive
    fn process_include(&mut self, tokens: &[Token], mut i: usize, result: &mut Vec<Token>) -> Result<usize> {
        i += 1; // Skip 'include'

        if i >= tokens.len() {
            return Err(preprocessor_error(
                &tokens[i - 1].location,
                "Unexpected end of file after #include",
            ));
        }

        let token = &tokens[i];

        let (filename, is_system) = match &token.kind {
            TokenKind::StringLiteral(name) => (name.clone(), false),
            TokenKind::LessThan => {
                // Parse <filename>
                i += 1;
                let mut filename = String::new();

                while i < tokens.len() && tokens[i].kind != TokenKind::GreaterThan {
                    if let TokenKind::Identifier(part) = &tokens[i].kind {
                        filename.push_str(part);
                    } else if let TokenKind::Dot = &tokens[i].kind {
                        filename.push('.');
                    } else if let TokenKind::Slash = &tokens[i].kind {
                        filename.push('/');
                    } else {
                        return Err(preprocessor_error(
                            &tokens[i].location,
                            "Invalid character in include filename",
                        ));
                    }
                    i += 1;
                }

                if i >= tokens.len() {
                    return Err(preprocessor_error(
                        &token.location,
                        "Unterminated include filename",
                    ));
                }

                i += 1; // Skip '>'
                (filename, true)
            }
            _ => {
                return Err(preprocessor_error(
                    &token.location,
                    "Expected filename after #include",
                ));
            }
        };

        // Find the file
        let file_path = if is_system {
            // Search in include paths
            let mut found_path = None;
            for path in &self.include_paths {
                let full_path = path.join(&filename);
                if full_path.exists() {
                    found_path = Some(full_path);
                    break;
                }
            }

            found_path.ok_or_else(|| {
                preprocessor_error(
                    &token.location,
                    format!("Cannot find include file: {}", filename),
                )
            })?
        } else {
            // Relative to current file
            let current_dir = Path::new(&token.filename).parent().unwrap_or_else(|| Path::new(""));
            let full_path = current_dir.join(&filename);

            if !full_path.exists() {
                return Err(preprocessor_error(
                    &token.location,
                    format!("Cannot find include file: {}", filename),
                ));
            }

            full_path
        };

        // Read and preprocess the included file
        let content = fs::read_to_string(&file_path).map_err(|e| {
            preprocessor_error(
                &token.location,
                format!("Failed to read include file: {}", e),
            )
        })?;

        let mut lexer = Lexer::new(&content, file_path.to_string_lossy().to_string());
        let included_tokens = lexer.tokenize()?;

        let preprocessed_tokens = self.preprocess(included_tokens)?;
        result.extend(preprocessed_tokens);

        // Skip to the next token
        i += 1;

        Ok(i)
    }
}
