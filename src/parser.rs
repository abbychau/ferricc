use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::{BinaryOp, Location, Node, Type, UnaryOp};
use crate::error::{syntax_error, Result};
use crate::lexer::{Token, TokenKind};

/// Parser for C source code
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    current: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let mut iter = tokens.iter().peekable();
        let current = iter.next();

        Self {
            tokens: iter,
            current,
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    /// Peek at the next token without advancing
    fn peek(&mut self) -> Option<&'a Token> {
        self.tokens.peek().copied()
    }

    /// Check if the current token matches the expected kind
    fn check(&self, kind: &TokenKind) -> bool {
        match self.current {
            Some(token) => &token.kind == kind,
            None => false,
        }
    }

    /// Consume the current token if it matches the expected kind
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume the current token if it matches the expected kind, otherwise return an error
    fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<&'a Token> {
        match self.current {
            Some(token) if &token.kind == kind => {
                let current = token;
                self.advance();
                Ok(current)
            }
            Some(token) => Err(syntax_error(
                &token.location,
                format!("{}, found {:?}", message, token.kind),
            )),
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                format!("{}, found end of file", message),
            )),
        }
    }

    /// Parse a program
    pub fn parse_program(&mut self) -> Result<Node> {
        let mut declarations = Vec::new();

        while self.current.is_some() && !self.check(&TokenKind::Eof) {
            declarations.push(self.parse_declaration()?);
        }

        Ok(Node::Program(declarations))
    }

    /// Parse a declaration
    fn parse_declaration(&mut self) -> Result<Node> {
        // Check for type specifiers
        if self.check(&TokenKind::Int) || self.check(&TokenKind::Char) ||
           self.check(&TokenKind::Void) || self.check(&TokenKind::Long) ||
           self.check(&TokenKind::Struct) {
            let type_ = self.parse_type()?;

            // Parse the identifier
            if let Some(token) = self.current {
                if let TokenKind::Identifier(name) = &token.kind {
                    let name = name.clone();
                    let location = token.location.clone();
                    self.advance(); // Consume the identifier

                    // Check if it's a function declaration or a variable declaration
                    if self.check(&TokenKind::LeftParen) {
                        self.parse_function_declaration(name, type_, location)
                    } else {
                        self.parse_variable_declaration(name, type_, location)
                    }
                } else {
                    Err(syntax_error(
                        &token.location,
                        format!("Expected identifier, found {:?}", token.kind),
                    ))
                }
            } else {
                Err(syntax_error(
                    &Location {
                        file: "unknown".to_string(),
                        line: 0,
                        column: 0,
                    },
                    "Unexpected end of file",
                ))
            }
        } else if let Some(token) = self.current {
            if let TokenKind::Identifier(name) = &token.kind {
                if name == "int" || name == "char" || name == "void" || name == "long" {
                    // Handle type names as identifiers
                    let type_ = match name.as_str() {
                        "int" => {
                            self.advance();
                            Type::Int
                        },
                        "char" => {
                            self.advance();
                            Type::Char
                        },
                        "void" => {
                            self.advance();
                            Type::Void
                        },
                        "long" => {
                            self.advance();
                            Type::Long
                        },
                        _ => unreachable!(),
                    };

                    // Parse the identifier
                    if let Some(token) = self.current {
                        if let TokenKind::Identifier(name) = &token.kind {
                            let name = name.clone();
                            let location = token.location.clone();
                            self.advance(); // Consume the identifier

                            // Check if it's a function declaration or a variable declaration
                            if self.check(&TokenKind::LeftParen) {
                                self.parse_function_declaration(name, type_, location)
                            } else {
                                self.parse_variable_declaration(name, type_, location)
                            }
                        } else {
                            Err(syntax_error(
                                &token.location,
                                format!("Expected identifier, found {:?}", token.kind),
                            ))
                        }
                    } else {
                        Err(syntax_error(
                            &Location {
                                file: "unknown".to_string(),
                                line: 0,
                                column: 0,
                            },
                            "Unexpected end of file",
                        ))
                    }
                } else {
                    Err(syntax_error(
                        &token.location,
                        "Expected declaration",
                    ))
                }
            } else {
                Err(syntax_error(
                    &token.location,
                    "Expected declaration",
                ))
            }
        } else {
            Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                "Expected declaration",
            ))
        }
    }

    /// Parse a type
    fn parse_type(&mut self) -> Result<Type> {
        let base_type = if self.match_token(&TokenKind::Void) {
            Type::Void
        } else if self.match_token(&TokenKind::Char) {
            Type::Char
        } else if self.match_token(&TokenKind::Int) {
            Type::Int
        } else if self.match_token(&TokenKind::Long) {
            Type::Long
        } else if self.match_token(&TokenKind::Struct) {
            // Parse struct type
            let name = if let Some(token) = self.current {
                if let TokenKind::Identifier(name) = &token.kind {
                    self.advance();
                    name.clone()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            // Parse struct body if present
            let members = if self.match_token(&TokenKind::LeftBrace) {
                let mut members = Vec::new();

                while !self.check(&TokenKind::RightBrace) && self.current.is_some() {
                    let member_type = self.parse_type()?;
                    let member_name = match &self.expect(&TokenKind::Identifier("".to_string()), "Expected member name")?.kind {
                        TokenKind::Identifier(name) => name.clone(),
                        _ => unreachable!(),
                    };

                    self.expect(&TokenKind::Semicolon, "Expected ';' after struct member")?;

                    members.push((member_name, member_type));
                }

                self.expect(&TokenKind::RightBrace, "Expected '}' after struct body")?;

                members
            } else {
                Vec::new()
            };

            Type::Struct(name, members)
        } else {
            return Err(syntax_error(
                &self.current.unwrap().location,
                "Expected type specifier",
            ));
        };

        // Handle pointers
        let mut type_ = base_type;
        while self.match_token(&TokenKind::Asterisk) {
            type_ = Type::Pointer(Box::new(type_));
        }

        Ok(type_)
    }

    /// Parse a variable declaration
    fn parse_variable_declaration(&mut self, name: String, type_: Type, location: Location) -> Result<Node> {
        let mut var_type = type_;

        // Handle array declarations
        if self.match_token(&TokenKind::LeftBracket) {
            let size = if let Some(token) = self.current {
                if let TokenKind::IntLiteral(size) = token.kind {
                    self.advance();
                    Some(size as usize)
                } else {
                    None
                }
            } else {
                None
            };

            self.expect(&TokenKind::RightBracket, "Expected ']' after array size")?;

            var_type = Type::Array(Box::new(var_type), size);
        }

        // Handle initializer
        let initializer = if self.match_token(&TokenKind::Assign) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(&TokenKind::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Node::VarDecl {
            name,
            type_: var_type,
            initializer,
            location,
        })
    }

    /// Parse a function declaration
    fn parse_function_declaration(&mut self, name: String, return_type: Type, location: Location) -> Result<Node> {
        // Special case for main function with command-line arguments
        let is_main = name == "main";
        self.expect(&TokenKind::LeftParen, "Expected '(' after function name")?;

        // Parse parameters
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_type = self.parse_type()?;

                if let Some(token) = self.current {
                    if let TokenKind::Identifier(name) = &token.kind {
                        let param_name = name.clone();
                        self.advance(); // Consume the identifier
                        params.push((param_name, param_type));
                    } else {
                        return Err(syntax_error(
                            &token.location,
                            format!("Expected parameter name, found {:?}", token.kind),
                        ));
                    }
                } else {
                    return Err(syntax_error(
                        &Location {
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                        },
                        "Unexpected end of file",
                    ));
                }

                // Parameter already added

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }

                // Check for variadic arguments (...)
                if self.match_token(&TokenKind::Ellipsis) {
                    // Add a special parameter for variadic arguments
                    params.push(("...".to_string(), Type::Void));
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse function body if present
        let body = if self.check(&TokenKind::LeftBrace) {
            Some(Box::new(self.parse_block()?))
        } else {
            self.expect(&TokenKind::Semicolon, "Expected ';' after function declaration")?;
            None
        };

        Ok(Node::FunctionDecl {
            name,
            return_type,
            params,
            body,
            location,
        })
    }

    /// Parse a block statement
    fn parse_block(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.expect(&TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.current.is_some() {
            statements.push(self.parse_statement()?);
        }

        self.expect(&TokenKind::RightBrace, "Expected '}'")?;

        Ok(Node::BlockStmt(statements, location))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Node> {
        match self.current {
            Some(token) => match &token.kind {
                TokenKind::If => self.parse_if_statement(),
                TokenKind::While => self.parse_while_statement(),
                TokenKind::For => self.parse_for_statement(),
                TokenKind::Return => self.parse_return_statement(),
                TokenKind::LeftBrace => self.parse_block(),
                TokenKind::Int | TokenKind::Char | TokenKind::Void | TokenKind::Long | TokenKind::Struct => {
                    let decl = self.parse_declaration()?;
                    Ok(decl)
                }
                TokenKind::Semicolon => {
                    self.advance();
                    Ok(Node::ExpressionStmt(Box::new(Node::IntLiteral(0, token.location.clone()))))
                }
                _ => {
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon, "Expected ';' after expression")?;
                    Ok(Node::ExpressionStmt(Box::new(expr)))
                }
            },
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                "Unexpected end of file",
            )),
        }
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'if'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "Expected ')' after condition")?;

        let then_branch = self.parse_statement()?;

        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Node::IfStmt {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            location,
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'while'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "Expected ')' after condition")?;

        let body = self.parse_statement()?;

        Ok(Node::WhileStmt {
            condition: Box::new(condition),
            body: Box::new(body),
            location,
        })
    }

    /// Parse a for statement
    fn parse_for_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'for'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'for'")?;

        // Parse initializer
        let init = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let init_expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after for initializer")?;
            Some(Box::new(init_expr))
        };

        // Parse condition
        let condition = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let cond_expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after for condition")?;
            Some(Box::new(cond_expr))
        };

        // Parse increment
        let increment = if self.match_token(&TokenKind::RightParen) {
            None
        } else {
            let inc_expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen, "Expected ')' after for increment")?;
            Some(Box::new(inc_expr))
        };

        let body = self.parse_statement()?;

        Ok(Node::ForStmt {
            init,
            condition,
            increment,
            body: Box::new(body),
            location,
        })
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'return'

        let value = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after return value")?;
            Some(Box::new(expr))
        };

        Ok(Node::ReturnStmt(value, location))
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Node> {
        self.parse_assignment()
    }

    /// Parse an assignment expression
    fn parse_assignment(&mut self) -> Result<Node> {
        let expr = self.parse_logical_or()?;

        if self.match_token(&TokenKind::Assign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            Ok(Node::BinaryExpr {
                op: BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                location,
            })
        } else {
            Ok(expr)
        }
    }

    /// Parse a logical OR expression
    fn parse_logical_or(&mut self) -> Result<Node> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::LogicalOr) {
            let location = self.current.unwrap().location.clone();
            let right = self.parse_logical_and()?;

            expr = Node::BinaryExpr {
                op: BinaryOp::LogicalOr,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a logical AND expression
    fn parse_logical_and(&mut self) -> Result<Node> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::LogicalAnd) {
            let location = self.current.unwrap().location.clone();
            let right = self.parse_equality()?;

            expr = Node::BinaryExpr {
                op: BinaryOp::LogicalAnd,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse an equality expression
    fn parse_equality(&mut self) -> Result<Node> {
        let mut expr = self.parse_relational()?;

        loop {
            let op = if self.match_token(&TokenKind::Equal) {
                BinaryOp::Equal
            } else if self.match_token(&TokenKind::NotEqual) {
                BinaryOp::NotEqual
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_relational()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a relational expression
    fn parse_relational(&mut self) -> Result<Node> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = if self.match_token(&TokenKind::LessThan) {
                BinaryOp::Less
            } else if self.match_token(&TokenKind::LessThanEqual) {
                BinaryOp::LessEqual
            } else if self.match_token(&TokenKind::GreaterThan) {
                BinaryOp::Greater
            } else if self.match_token(&TokenKind::GreaterThanEqual) {
                BinaryOp::GreaterEqual
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_additive()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse an additive expression
    fn parse_additive(&mut self) -> Result<Node> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else if self.match_token(&TokenKind::Minus) {
                BinaryOp::Subtract
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_multiplicative()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a multiplicative expression
    fn parse_multiplicative(&mut self) -> Result<Node> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = if self.match_token(&TokenKind::Asterisk) {
                BinaryOp::Multiply
            } else if self.match_token(&TokenKind::Slash) {
                BinaryOp::Divide
            } else if self.match_token(&TokenKind::Percent) {
                BinaryOp::Modulo
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_unary()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a unary expression
    fn parse_unary(&mut self) -> Result<Node> {
        if let Some(token) = self.current {
            let op = match token.kind {
                TokenKind::Minus => {
                    self.advance();
                    Some(UnaryOp::Negate)
                }
                TokenKind::LogicalNot => {
                    self.advance();
                    Some(UnaryOp::LogicalNot)
                }
                TokenKind::BitwiseNot => {
                    self.advance();
                    Some(UnaryOp::BitwiseNot)
                }
                TokenKind::Asterisk => {
                    self.advance();
                    Some(UnaryOp::Dereference)
                }
                TokenKind::BitwiseAnd => {
                    self.advance();
                    Some(UnaryOp::AddressOf)
                }
                _ => None,
            };

            if let Some(op) = op {
                let location = token.location.clone();
                let expr = self.parse_unary()?;

                return Ok(Node::UnaryExpr {
                    op,
                    expr: Box::new(expr),
                    location,
                });
            }
        }

        self.parse_postfix()
    }

    /// Parse a postfix expression
    fn parse_postfix(&mut self) -> Result<Node> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenKind::LeftParen) {
                // Function call
                let location = self.current.unwrap().location.clone();
                let mut args = Vec::new();

                if !self.check(&TokenKind::RightParen) {
                    loop {
                        args.push(self.parse_expression()?);

                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }

                self.expect(&TokenKind::RightParen, "Expected ')' after arguments")?;

                if let Node::Identifier(name, _) = expr {
                    expr = Node::FunctionCall {
                        name,
                        args,
                        location,
                    };
                } else {
                    return Err(syntax_error(
                        &location,
                        "Expected function name before '('",
                    ));
                }
            } else if self.match_token(&TokenKind::LeftBracket) {
                // Array access
                let location = self.current.unwrap().location.clone();
                let index = self.parse_expression()?;

                self.expect(&TokenKind::RightBracket, "Expected ']' after index")?;

                // Array access is equivalent to *(array + index)
                let array_plus_index = Node::BinaryExpr {
                    op: BinaryOp::Add,
                    left: Box::new(expr),
                    right: Box::new(index),
                    location: location.clone(),
                };

                expr = Node::UnaryExpr {
                    op: UnaryOp::Dereference,
                    expr: Box::new(array_plus_index),
                    location,
                };
            } else if self.match_token(&TokenKind::Dot) {
                // Struct member access
                let location = self.current.unwrap().location.clone();
                let _member = match &self.expect(&TokenKind::Identifier("".to_string()), "Expected member name after '.'")?
                    .kind
                {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };

                // TODO: Implement struct member access
                return Err(syntax_error(
                    &location,
                    "Struct member access not implemented yet",
                ));
            } else if self.match_token(&TokenKind::Arrow) {
                // Struct pointer member access
                let location = self.current.unwrap().location.clone();
                let _member = match &self.expect(&TokenKind::Identifier("".to_string()), "Expected member name after '->'")?
                    .kind
                {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };

                // TODO: Implement struct pointer member access
                return Err(syntax_error(
                    &location,
                    "Struct pointer member access not implemented yet",
                ));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse a primary expression
    fn parse_primary(&mut self) -> Result<Node> {
        match self.current {
            Some(token) => {
                let location = token.location.clone();

                match &token.kind {
                    TokenKind::IntLiteral(value) => {
                        self.advance();
                        Ok(Node::IntLiteral(*value, location))
                    }
                    TokenKind::CharLiteral(value) => {
                        self.advance();
                        Ok(Node::CharLiteral(*value, location))
                    }
                    TokenKind::StringLiteral(value) => {
                        self.advance();
                        Ok(Node::StringLiteral(value.clone(), location))
                    }
                    TokenKind::Identifier(name) => {
                        self.advance();
                        Ok(Node::Identifier(name.clone(), location))
                    }
                    TokenKind::LeftParen => {
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.expect(&TokenKind::RightParen, "Expected ')' after expression")?;
                        Ok(expr)
                    }
                    _ => Err(syntax_error(
                        &location,
                        format!("Unexpected token: {:?}", token.kind),
                    )),
                }
            }
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                "Unexpected end of file",
            )),
        }
    }
}
