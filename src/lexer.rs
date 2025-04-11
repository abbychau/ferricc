use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use lazy_static::lazy_static;

use crate::ast::Location;
use crate::error::{lexical_error, Result};

/// Represents a token in the C language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,

    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    CharLiteral(char),
    StringLiteral(String),

    // Operators
    Plus,           // +
    Minus,          // -
    Asterisk,       // *
    Slash,          // /
    Percent,        // %
    Increment,      // ++
    Decrement,      // --
    Equal,          // ==
    NotEqual,       // !=
    LessThan,       // <
    LessThanEqual,  // <=
    GreaterThan,    // >
    GreaterThanEqual, // >=
    LogicalAnd,     // &&
    LogicalOr,      // ||
    LogicalNot,     // !
    BitwiseAnd,     // &
    BitwiseOr,      // |
    BitwiseXor,     // ^
    BitwiseNot,     // ~
    ShiftLeft,      // <<
    ShiftRight,     // >>
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModuloAssign,   // %=
    AndAssign,      // &=
    OrAssign,       // |=
    XorAssign,      // ^=
    ShiftLeftAssign, // <<=
    ShiftRightAssign, // >>=

    // Punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Semicolon,      // ;
    Comma,          // ,
    Dot,            // .
    Arrow,          // ->
    Colon,          // :
    QuestionMark,   // ?
    Ellipsis,       // ...

    // Preprocessor
    Hash,           // #
    HashHash,       // ##

    // End of file
    Eof,
}

/// Represents a token with its location in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
    pub filename: String,
    pub at_bol: bool,  // Beginning of line
}

impl Token {
    pub fn new(kind: TokenKind, location: Location) -> Self {
        Self {
            kind,
            location: location.clone(),
            filename: location.file.clone(),
            at_bol: false,
        }
    }

    pub fn with_at_bol(mut self, at_bol: bool) -> Self {
        self.at_bol = at_bol;
        self
    }
}

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = {
        let mut m = HashMap::new();
        m.insert("auto", TokenKind::Auto);
        m.insert("break", TokenKind::Break);
        m.insert("case", TokenKind::Case);
        m.insert("char", TokenKind::Char);
        m.insert("const", TokenKind::Const);
        m.insert("continue", TokenKind::Continue);
        m.insert("default", TokenKind::Default);
        m.insert("do", TokenKind::Do);
        m.insert("double", TokenKind::Double);
        m.insert("else", TokenKind::Else);
        m.insert("enum", TokenKind::Enum);
        m.insert("extern", TokenKind::Extern);
        m.insert("float", TokenKind::Float);
        m.insert("for", TokenKind::For);
        m.insert("goto", TokenKind::Goto);
        m.insert("if", TokenKind::If);
        m.insert("int", TokenKind::Int);
        m.insert("long", TokenKind::Long);
        m.insert("register", TokenKind::Register);
        m.insert("return", TokenKind::Return);
        m.insert("short", TokenKind::Short);
        m.insert("signed", TokenKind::Signed);
        m.insert("sizeof", TokenKind::Sizeof);
        m.insert("static", TokenKind::Static);
        m.insert("struct", TokenKind::Struct);
        m.insert("switch", TokenKind::Switch);
        m.insert("typedef", TokenKind::Typedef);
        m.insert("union", TokenKind::Union);
        m.insert("unsigned", TokenKind::Unsigned);
        m.insert("void", TokenKind::Void);
        m.insert("volatile", TokenKind::Volatile);
        m.insert("while", TokenKind::While);
        m
    };
}

/// Lexer for C source code
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    filename: String,
    line: usize,
    column: usize,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, filename: String) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.next();

        Self {
            input: chars,
            filename,
            line: 1,
            column: 1,
            current_char,
        }
    }

    /// Get the current location in the source code
    fn location(&self) -> Location {
        Location {
            file: self.filename.clone(),
            line: self.line,
            column: self.column,
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some(c) = self.current_char {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.current_char = self.input.next();
    }

    /// Peek at the next character without advancing
    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    /// Skip a comment (either // or /* */)
    fn skip_comment(&mut self) -> Result<()> {
        if self.current_char == Some('/') {
            if let Some(next) = self.peek() {
                if next == '/' {
                    // Line comment
                    self.advance(); // Skip the second '/'
                    self.advance();

                    while let Some(c) = self.current_char {
                        if c == '\n' {
                            self.advance();
                            return Ok(());
                        }
                        self.advance();
                    }
                    return Ok(());
                } else if next == '*' {
                    // Block comment
                    self.advance(); // Skip the '*'
                    self.advance();

                    let start_location = self.location();

                    while let Some(c) = self.current_char {
                        if c == '*' && self.peek() == Some('/') {
                            self.advance(); // Skip the '*'
                            self.advance(); // Skip the '/'
                            return Ok(());
                        }
                        self.advance();
                    }

                    return Err(lexical_error(
                        &start_location,
                        "Unterminated block comment",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Tokenize an identifier or keyword
    fn identifier(&mut self) -> Result<Token> {
        let start_location = self.location();
        let mut identifier = String::new();

        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let token_kind = if let Some(keyword) = KEYWORDS.get(identifier.as_str()) {
            keyword.clone()
        } else {
            TokenKind::Identifier(identifier)
        };

        Ok(Token::new(token_kind, start_location))
    }

    /// Tokenize a number literal
    fn number(&mut self) -> Result<Token> {
        let start_location = self.location();
        let mut number = String::new();

        // Check for hexadecimal, octal, or decimal
        if self.current_char == Some('0') {
            number.push('0');
            self.advance();

            if let Some(c) = self.current_char {
                if c == 'x' || c == 'X' {
                    // Hexadecimal
                    number.push(c);
                    self.advance();

                    while let Some(c) = self.current_char {
                        if c.is_digit(16) {
                            number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    let value = i64::from_str_radix(&number[2..], 16)
                        .map_err(|_| lexical_error(&start_location, "Invalid hexadecimal literal"))?;

                    return Ok(Token::new(TokenKind::IntLiteral(value), start_location));
                } else if c.is_digit(8) {
                    // Octal
                    while let Some(c) = self.current_char {
                        if c.is_digit(8) {
                            number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    let value = i64::from_str_radix(&number, 8)
                        .map_err(|_| lexical_error(&start_location, "Invalid octal literal"))?;

                    return Ok(Token::new(TokenKind::IntLiteral(value), start_location));
                }
            }
        }

        // Decimal
        while let Some(c) = self.current_char {
            if c.is_digit(10) {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let value = number
            .parse::<i64>()
            .map_err(|_| lexical_error(&start_location, "Invalid integer literal"))?;

        Ok(Token::new(TokenKind::IntLiteral(value), start_location))
    }

    /// Tokenize a character literal
    fn char_literal(&mut self) -> Result<Token> {
        let start_location = self.location();
        self.advance(); // Skip the opening quote

        let c = match self.current_char {
            Some('\\') => {
                self.advance();
                match self.current_char {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('\"') => '\"',
                    Some('0') => '\0',
                    Some(c) => return Err(lexical_error(
                        &self.location(),
                        format!("Unknown escape sequence: \\{}", c),
                    )),
                    None => return Err(lexical_error(
                        &self.location(),
                        "Unterminated character literal",
                    )),
                }
            }
            Some(c) => c,
            None => return Err(lexical_error(
                &self.location(),
                "Unterminated character literal",
            )),
        };

        self.advance(); // Skip the character

        if self.current_char != Some('\'') {
            return Err(lexical_error(
                &self.location(),
                "Expected closing quote for character literal",
            ));
        }

        self.advance(); // Skip the closing quote

        Ok(Token::new(TokenKind::CharLiteral(c), start_location))
    }

    /// Tokenize a string literal
    fn string_literal(&mut self) -> Result<Token> {
        let start_location = self.location();
        self.advance(); // Skip the opening quote

        let mut string = String::new();

        while let Some(c) = self.current_char {
            if c == '"' {
                self.advance(); // Skip the closing quote
                return Ok(Token::new(TokenKind::StringLiteral(string), start_location));
            } else if c == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('\'') => string.push('\''),
                    Some('\"') => string.push('\"'),
                    Some('0') => string.push('\0'),
                    Some(c) => return Err(lexical_error(
                        &self.location(),
                        format!("Unknown escape sequence: \\{}", c),
                    )),
                    None => return Err(lexical_error(
                        &self.location(),
                        "Unterminated string literal",
                    )),
                }
            } else {
                string.push(c);
            }
            self.advance();
        }

        Err(lexical_error(
            &start_location,
            "Unterminated string literal",
        ))
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        if let Some(c) = self.current_char {
            let location = self.location();

            match c {
                // End of file
                '\0' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Eof, location))
                }

                // Identifiers and keywords
                c if c.is_alphabetic() || c == '_' => self.identifier(),

                // Number literals
                c if c.is_digit(10) => self.number(),

                // Character literals
                '\'' => self.char_literal(),

                // String literals
                '"' => self.string_literal(),

                // Comments and operators
                '/' => {
                    if let Some(next) = self.peek() {
                        match next {
                            '/' | '*' => {
                                self.skip_comment()?;
                                self.next_token()
                            }
                            '=' => {
                                self.advance();
                                self.advance();
                                Ok(Token::new(TokenKind::DivideAssign, location))
                            }
                            _ => {
                                self.advance();
                                Ok(Token::new(TokenKind::Slash, location))
                            }
                        }
                    } else {
                        self.advance();
                        Ok(Token::new(TokenKind::Slash, location))
                    }
                }

                // Operators and punctuation
                '+' => {
                    self.advance();
                    match self.current_char {
                        Some('+') => {
                            self.advance();
                            Ok(Token::new(TokenKind::Increment, location))
                        }
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::PlusAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::Plus, location)),
                    }
                }
                '-' => {
                    self.advance();
                    match self.current_char {
                        Some('-') => {
                            self.advance();
                            Ok(Token::new(TokenKind::Decrement, location))
                        }
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::MinusAssign, location))
                        }
                        Some('>') => {
                            self.advance();
                            Ok(Token::new(TokenKind::Arrow, location))
                        }
                        _ => Ok(Token::new(TokenKind::Minus, location)),
                    }
                }
                '*' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::MultiplyAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::Asterisk, location)),
                    }
                }
                '%' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::ModuloAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::Percent, location)),
                    }
                }
                '=' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::Equal, location))
                        }
                        _ => Ok(Token::new(TokenKind::Assign, location)),
                    }
                }
                '!' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::NotEqual, location))
                        }
                        _ => Ok(Token::new(TokenKind::LogicalNot, location)),
                    }
                }
                '<' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::LessThanEqual, location))
                        }
                        Some('<') => {
                            self.advance();
                            match self.current_char {
                                Some('=') => {
                                    self.advance();
                                    Ok(Token::new(TokenKind::ShiftLeftAssign, location))
                                }
                                _ => Ok(Token::new(TokenKind::ShiftLeft, location)),
                            }
                        }
                        _ => Ok(Token::new(TokenKind::LessThan, location)),
                    }
                }
                '>' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::GreaterThanEqual, location))
                        }
                        Some('>') => {
                            self.advance();
                            match self.current_char {
                                Some('=') => {
                                    self.advance();
                                    Ok(Token::new(TokenKind::ShiftRightAssign, location))
                                }
                                _ => Ok(Token::new(TokenKind::ShiftRight, location)),
                            }
                        }
                        _ => Ok(Token::new(TokenKind::GreaterThan, location)),
                    }
                }
                '&' => {
                    self.advance();
                    match self.current_char {
                        Some('&') => {
                            self.advance();
                            Ok(Token::new(TokenKind::LogicalAnd, location))
                        }
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::AndAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::BitwiseAnd, location)),
                    }
                }
                '|' => {
                    self.advance();
                    match self.current_char {
                        Some('|') => {
                            self.advance();
                            Ok(Token::new(TokenKind::LogicalOr, location))
                        }
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::OrAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::BitwiseOr, location)),
                    }
                }
                '^' => {
                    self.advance();
                    match self.current_char {
                        Some('=') => {
                            self.advance();
                            Ok(Token::new(TokenKind::XorAssign, location))
                        }
                        _ => Ok(Token::new(TokenKind::BitwiseXor, location)),
                    }
                }
                '~' => {
                    self.advance();
                    Ok(Token::new(TokenKind::BitwiseNot, location))
                }

                // Punctuation
                '(' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LeftParen, location))
                }
                ')' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RightParen, location))
                }
                '{' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LeftBrace, location))
                }
                '}' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RightBrace, location))
                }
                '[' => {
                    self.advance();
                    Ok(Token::new(TokenKind::LeftBracket, location))
                }
                ']' => {
                    self.advance();
                    Ok(Token::new(TokenKind::RightBracket, location))
                }
                ';' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Semicolon, location))
                }
                ',' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Comma, location))
                }
                '.' => {
                    self.advance();
                    if self.current_char == Some('.') && self.peek() == Some('.') {
                        self.advance();
                        self.advance();
                        Ok(Token::new(TokenKind::Ellipsis, location))
                    } else {
                        Ok(Token::new(TokenKind::Dot, location))
                    }
                }
                ':' => {
                    self.advance();
                    Ok(Token::new(TokenKind::Colon, location))
                }
                '?' => {
                    self.advance();
                    Ok(Token::new(TokenKind::QuestionMark, location))
                }
                '#' => {
                    self.advance();
                    if self.current_char == Some('#') {
                        self.advance();
                        Ok(Token::new(TokenKind::HashHash, location))
                    } else {
                        Ok(Token::new(TokenKind::Hash, location))
                    }
                }

                // Unknown character
                _ => {
                    self.advance();
                    Err(lexical_error(
                        &location,
                        format!("Unexpected character: {}", c),
                    ))
                }
            }
        } else {
            // End of file
            Ok(Token::new(TokenKind::Eof, self.location()))
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}
