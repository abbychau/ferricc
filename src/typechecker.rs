use std::collections::HashMap;

use crate::ast::{BinaryOp, Node, Type, UnaryOp};
use crate::error::{type_error, Result};

/// Symbol table for tracking variables and their types
#[derive(Debug, Clone)]
struct SymbolTable {
    scopes: Vec<HashMap<String, Type>>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Enter a new scope
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable in the current scope
    fn define(&mut self, name: &str, type_: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), type_);
        }
    }

    /// Look up a variable in all scopes, starting from the innermost
    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(type_) = scope.get(name) {
                return Some(type_.clone());
            }
        }
        None
    }
}

/// Type checker for C source code
pub struct TypeChecker {
    symbol_table: SymbolTable,
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_function_return_type: None,
        }
    }

    /// Check if two types are compatible
    fn is_compatible(&self, left: &Type, right: &Type) -> bool {
        match (left, right) {
            (Type::Void, Type::Void) => true,
            (Type::Char, Type::Char) => true,
            (Type::Int, Type::Int) => true,
            (Type::Long, Type::Long) => true,
            (Type::Int, Type::Char) | (Type::Char, Type::Int) => true,
            (Type::Long, Type::Int) | (Type::Int, Type::Long) => true,
            (Type::Long, Type::Char) | (Type::Char, Type::Long) => true,
            (Type::Pointer(l), Type::Pointer(r)) => self.is_compatible(l, r),
            (Type::Array(l, _), Type::Array(r, _)) => self.is_compatible(l, r),
            (Type::Array(l, _), Type::Pointer(r)) | (Type::Pointer(l), Type::Array(r, _)) => {
                self.is_compatible(l, r)
            }
            (Type::Function(l_ret, l_params, l_variadic), Type::Function(r_ret, r_params, r_variadic)) => {
                self.is_compatible(l_ret, r_ret)
                    && l_params.len() == r_params.len()
                    && l_variadic == r_variadic
                    && l_params
                        .iter()
                        .zip(r_params.iter())
                        .all(|(l, r)| self.is_compatible(l, r))
            }
            _ => false,
        }
    }

    /// Check if a type is an integer type
    fn is_integer_type(&self, type_: &Type) -> bool {
        matches!(type_, Type::Char | Type::Int | Type::Long)
    }

    /// Check if a type is a pointer type
    fn is_pointer_type(&self, type_: &Type) -> bool {
        matches!(type_, Type::Pointer(_) | Type::Array(_, _))
    }

    /// Type check a program
    pub fn check_program(&mut self, program: &Node) -> Result<()> {
        match program {
            Node::Program(declarations) => {
                for decl in declarations {
                    self.check_node(decl)?;
                }
                Ok(())
            }
            _ => panic!("Expected program node"),
        }
    }

    /// Type check a node
    fn check_node(&mut self, node: &Node) -> Result<Type> {
        match node {
            Node::IntLiteral(_, _) => Ok(Type::Int),
            Node::CharLiteral(_, _) => Ok(Type::Char),
            Node::StringLiteral(_, _location) => {
                Ok(Type::Pointer(Box::new(Type::Char)))
            }
            Node::Identifier(name, location) => {
                if let Some(type_) = self.symbol_table.lookup(name) {
                    Ok(type_)
                } else {
                    Err(type_error(
                        &location,
                        format!("Undefined variable: {}", name),
                    ))
                }
            }
            Node::BinaryExpr {
                op,
                left,
                right,
                location,
            } => {
                let left_type = self.check_node(left)?;
                let right_type = self.check_node(right)?;

                match op {
                    BinaryOp::Add => {
                        if self.is_integer_type(&left_type) && self.is_integer_type(&right_type) {
                            // Integer addition
                            if matches!(left_type, Type::Long) || matches!(right_type, Type::Long) {
                                Ok(Type::Long)
                            } else {
                                Ok(Type::Int)
                            }
                        } else if self.is_pointer_type(&left_type) && self.is_integer_type(&right_type) {
                            // Pointer arithmetic
                            Ok(left_type)
                        } else if self.is_integer_type(&left_type) && self.is_pointer_type(&right_type) {
                            // Pointer arithmetic
                            Ok(right_type)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for addition: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::Subtract => {
                        if self.is_integer_type(&left_type) && self.is_integer_type(&right_type) {
                            // Integer subtraction
                            if matches!(left_type, Type::Long) || matches!(right_type, Type::Long) {
                                Ok(Type::Long)
                            } else {
                                Ok(Type::Int)
                            }
                        } else if self.is_pointer_type(&left_type) && self.is_integer_type(&right_type) {
                            // Pointer arithmetic
                            Ok(left_type)
                        } else if self.is_pointer_type(&left_type) && self.is_pointer_type(&right_type) {
                            // Pointer subtraction
                            Ok(Type::Int)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for subtraction: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                        if self.is_integer_type(&left_type) && self.is_integer_type(&right_type) {
                            // Integer multiplication/division/modulo
                            if matches!(left_type, Type::Long) || matches!(right_type, Type::Long) {
                                Ok(Type::Long)
                            } else {
                                Ok(Type::Int)
                            }
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for arithmetic operation: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if self.is_compatible(&left_type, &right_type) {
                            Ok(Type::Int)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for comparison: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                        if (self.is_integer_type(&left_type) && self.is_integer_type(&right_type))
                            || (self.is_pointer_type(&left_type) && self.is_pointer_type(&right_type))
                        {
                            Ok(Type::Int)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for comparison: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                        // Any type can be used in logical operations
                        Ok(Type::Int)
                    }
                    BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor | BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                        if self.is_integer_type(&left_type) && self.is_integer_type(&right_type) {
                            if matches!(left_type, Type::Long) || matches!(right_type, Type::Long) {
                                Ok(Type::Long)
                            } else {
                                Ok(Type::Int)
                            }
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Invalid operands for bitwise operation: {:?} and {:?}",
                                    left_type, right_type
                                ),
                            ))
                        }
                    }
                    BinaryOp::Assign => {
                        if self.is_compatible(&left_type, &right_type) {
                            Ok(left_type)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Cannot assign value of type {:?} to variable of type {:?}",
                                    right_type, left_type
                                ),
                            ))
                        }
                    }
                }
            }
            Node::UnaryExpr {
                op,
                expr,
                location,
            } => {
                let expr_type = self.check_node(expr)?;

                match op {
                    UnaryOp::Negate => {
                        if self.is_integer_type(&expr_type) {
                            Ok(expr_type)
                        } else {
                            Err(type_error(
                                &location,
                                format!("Cannot negate non-integer type: {:?}", expr_type),
                            ))
                        }
                    }
                    UnaryOp::LogicalNot => {
                        // Any type can be used with logical not
                        Ok(Type::Int)
                    }
                    UnaryOp::BitwiseNot => {
                        if self.is_integer_type(&expr_type) {
                            Ok(expr_type)
                        } else {
                            Err(type_error(
                                &location,
                                format!("Cannot apply bitwise not to non-integer type: {:?}", expr_type),
                            ))
                        }
                    }
                    UnaryOp::Dereference => {
                        if let Type::Pointer(inner) = expr_type {
                            Ok(*inner)
                        } else if let Type::Array(inner, _) = expr_type {
                            Ok(*inner)
                        } else {
                            Err(type_error(
                                &location,
                                format!("Cannot dereference non-pointer type: {:?}", expr_type),
                            ))
                        }
                    }
                    UnaryOp::AddressOf => {
                        Ok(Type::Pointer(Box::new(expr_type)))
                    }
                }
            }
            Node::FunctionCall {
                name,
                args,
                location,
            } => {
                if let Some(func_type) = self.symbol_table.lookup(name) {
                    if let Type::Function(return_type, param_types, is_variadic) = func_type {
                        if !is_variadic && args.len() != param_types.len() {
                            return Err(type_error(
                                &location,
                                format!(
                                    "Function {} expects {} arguments, but {} were provided",
                                    name,
                                    param_types.len(),
                                    args.len()
                                ),
                            ));
                        }

                        // Check arguments up to the number of fixed parameters
                        let check_count = param_types.len().min(args.len());
                        for i in 0..check_count {
                            let arg = &args[i];
                            let param_type = &param_types[i];
                            let arg_type = self.check_node(arg)?;
                            if !self.is_compatible(&arg_type, param_type) {
                                return Err(type_error(
                                    &location,
                                    format!(
                                        "Argument {} has type {:?}, but function {} expects {:?}",
                                        i + 1,
                                        arg_type,
                                        name,
                                        param_type
                                    ),
                                ));
                            }
                        }

                        Ok(*return_type)
                    } else {
                        Err(type_error(
                            &location,
                            format!("{} is not a function", name),
                        ))
                    }
                } else {
                    Err(type_error(
                        &location,
                        format!("Undefined function: {}", name),
                    ))
                }
            }
            Node::ExpressionStmt(expr) => {
                self.check_node(expr)?;
                Ok(Type::Void)
            }
            Node::ReturnStmt(value, location) => {
                let current_return_type = match &self.current_function_return_type {
                    Some(rt) => rt.clone(),
                    None => return Err(type_error(
                        &location,
                        "Return statement outside of function",
                    )),
                };

                match value {
                    Some(expr) => {
                        let expr_type = self.check_node(expr)?;
                        if self.is_compatible(&expr_type, &current_return_type) {
                            Ok(Type::Void)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Cannot return value of type {:?} from function with return type {:?}",
                                    expr_type, current_return_type
                                ),
                            ))
                        }
                    }
                    None => {
                        if matches!(current_return_type, Type::Void) {
                            Ok(Type::Void)
                        } else {
                            Err(type_error(
                                &location,
                                format!(
                                    "Cannot return void from function with return type {:?}",
                                    current_return_type
                                ),
                            ))
                        }
                    }
                }
            }
            Node::IfStmt {
                condition,
                then_branch,
                else_branch,
                location: _,
            } => {
                self.check_node(condition)?;

                self.symbol_table.enter_scope();
                self.check_node(then_branch)?;
                self.symbol_table.exit_scope();

                if let Some(else_branch) = else_branch {
                    self.symbol_table.enter_scope();
                    self.check_node(else_branch)?;
                    self.symbol_table.exit_scope();
                }

                Ok(Type::Void)
            }
            Node::WhileStmt {
                condition,
                body,
                location: _,
            } => {
                self.check_node(condition)?;

                self.symbol_table.enter_scope();
                self.check_node(body)?;
                self.symbol_table.exit_scope();

                Ok(Type::Void)
            }
            Node::ForStmt {
                init,
                condition,
                increment,
                body,
                location: _,
            } => {
                self.symbol_table.enter_scope();

                if let Some(init) = init {
                    self.check_node(init)?;
                }

                if let Some(condition) = condition {
                    self.check_node(condition)?;
                }

                if let Some(increment) = increment {
                    self.check_node(increment)?;
                }

                self.check_node(body)?;

                self.symbol_table.exit_scope();

                Ok(Type::Void)
            }
            Node::BlockStmt(statements, _) => {
                self.symbol_table.enter_scope();

                for stmt in statements {
                    self.check_node(stmt)?;
                }

                self.symbol_table.exit_scope();

                Ok(Type::Void)
            }
            Node::VarDecl {
                name,
                type_,
                initializer,
                location,
            } => {
                if let Some(init) = initializer {
                    let init_type = self.check_node(init)?;
                    if !self.is_compatible(&init_type, type_) {
                        return Err(type_error(
                            &location,
                            format!(
                                "Cannot initialize variable of type {:?} with value of type {:?}",
                                type_, init_type
                            ),
                        ));
                    }
                }

                self.symbol_table.define(name, type_.clone());

                Ok(Type::Void)
            }
            Node::FunctionDecl {
                name,
                return_type,
                params,
                body,
                location: _,
            } => {
                // This line is no longer needed as we filter out variadic parameters below
                // let param_types: Vec<Type> = params.iter().map(|(_, t)| t.clone()).collect();
                // Check if the function is variadic (has ... in parameters)
                let is_variadic = params.iter().any(|(name, _)| name == "...");

                // Remove the ... parameter if present
                let param_types: Vec<Type> = params.iter()
                    .filter(|(name, _)| name != "...")
                    .map(|(_, t)| t.clone())
                    .collect();

                let func_type = Type::Function(Box::new(return_type.clone()), param_types, is_variadic);

                self.symbol_table.define(name, func_type);

                if let Some(body) = body {
                    let prev_return_type = self.current_function_return_type.clone();
                    self.current_function_return_type = Some(return_type.clone());

                    self.symbol_table.enter_scope();

                    for (param_name, param_type) in params {
                        self.symbol_table.define(param_name, param_type.clone());
                    }

                    self.check_node(body)?;

                    self.symbol_table.exit_scope();

                    self.current_function_return_type = prev_return_type;
                }

                Ok(Type::Void)
            }
            Node::Program(_) => {
                panic!("Nested program node");
            }
        }
    }
}
