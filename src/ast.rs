use std::fmt;

/// Represents a location in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Represents a binary operator
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Equal,    // ==
    NotEqual, // !=
    Less,     // <
    LessEqual, // <=
    Greater,  // >
    GreaterEqual, // >=
    LogicalAnd, // &&
    LogicalOr,  // ||
    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    ShiftLeft,  // <<
    ShiftRight, // >>
    Assign,     // =
}

/// Represents a unary operator
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,    // -
    LogicalNot, // !
    BitwiseNot, // ~
    Dereference, // *
    AddressOf,   // &
}

/// Represents a C type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Char,
    Int,
    Long,
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Function(Box<Type>, Vec<Type>, bool), // Return type, parameter types, is_variadic
    Struct(String, Vec<(String, Type)>),
}

/// Represents an AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // Expressions
    IntLiteral(i64, Location),
    CharLiteral(char, Location),
    StringLiteral(String, Location),
    Identifier(String, Location),
    BinaryExpr {
        op: BinaryOp,
        left: Box<Node>,
        right: Box<Node>,
        location: Location,
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Node>,
        location: Location,
    },
    FunctionCall {
        name: String,
        args: Vec<Node>,
        location: Location,
    },

    // Statements
    ExpressionStmt(Box<Node>),
    ReturnStmt(Option<Box<Node>>, Location),
    IfStmt {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
        location: Location,
    },
    WhileStmt {
        condition: Box<Node>,
        body: Box<Node>,
        location: Location,
    },
    ForStmt {
        init: Option<Box<Node>>,
        condition: Option<Box<Node>>,
        increment: Option<Box<Node>>,
        body: Box<Node>,
        location: Location,
    },
    BlockStmt(Vec<Node>, Location),

    // Declarations
    VarDecl {
        name: String,
        type_: Type,
        initializer: Option<Box<Node>>,
        location: Location,
    },
    FunctionDecl {
        name: String,
        return_type: Type,
        params: Vec<(String, Type)>,
        body: Option<Box<Node>>,
        location: Location,
    },

    // Program
    Program(Vec<Node>),
}
