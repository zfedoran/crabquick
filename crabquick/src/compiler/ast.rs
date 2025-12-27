//! Abstract Syntax Tree (AST) node types
//!
//! Represents the structure of JavaScript programs after parsing.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use super::lexer::SourceLocation;

/// AST node ID for tracking
pub type NodeId = u32;

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    NotEq,
    StrictEq,
    StrictNotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // Logical
    LogicalAnd,
    LogicalOr,
    NullishCoalescing,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    UnsignedRightShift,

    // Special
    In,
    InstanceOf,
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
    TypeOf,
    Void,
    Delete,
}

/// Update operator (++ or --)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOp {
    Inc,
    Dec,
}

/// Assignment operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
}

/// Expression node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value
    Literal(Literal, SourceLocation),

    /// Identifier reference
    Identifier(String, SourceLocation),

    /// this
    This(SourceLocation),

    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        loc: SourceLocation,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        arg: Box<Expr>,
        prefix: bool,
        loc: SourceLocation,
    },

    /// Update operation (++/--)
    Update {
        op: UpdateOp,
        arg: Box<Expr>,
        prefix: bool,
        loc: SourceLocation,
    },

    /// Assignment
    Assignment {
        op: AssignOp,
        left: Box<Expr>,
        right: Box<Expr>,
        loc: SourceLocation,
    },

    /// Conditional (ternary) operator
    Conditional {
        test: Box<Expr>,
        consequent: Box<Expr>,
        alternate: Box<Expr>,
        loc: SourceLocation,
    },

    /// Function call
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        loc: SourceLocation,
    },

    /// new expression
    New {
        callee: Box<Expr>,
        args: Vec<Expr>,
        loc: SourceLocation,
    },

    /// Member expression (obj.prop or obj[prop])
    Member {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool, // true for [], false for .
        loc: SourceLocation,
    },

    /// Sequence expression (comma operator)
    Sequence {
        exprs: Vec<Expr>,
        loc: SourceLocation,
    },

    /// Array literal
    Array {
        elements: Vec<Option<Expr>>, // None for holes
        loc: SourceLocation,
    },

    /// Object literal
    Object {
        properties: Vec<Property>,
        loc: SourceLocation,
    },

    /// Function expression
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
        loc: SourceLocation,
    },

    /// Arrow function
    Arrow {
        params: Vec<String>,
        body: ArrowBody,
        loc: SourceLocation,
    },
}

impl Expr {
    /// Returns the source location of this expression
    pub fn location(&self) -> SourceLocation {
        match self {
            Expr::Literal(_, loc) |
            Expr::Identifier(_, loc) |
            Expr::This(loc) |
            Expr::Binary { loc, .. } |
            Expr::Unary { loc, .. } |
            Expr::Update { loc, .. } |
            Expr::Assignment { loc, .. } |
            Expr::Conditional { loc, .. } |
            Expr::Call { loc, .. } |
            Expr::New { loc, .. } |
            Expr::Member { loc, .. } |
            Expr::Sequence { loc, .. } |
            Expr::Array { loc, .. } |
            Expr::Object { loc, .. } |
            Expr::Function { loc, .. } |
            Expr::Arrow { loc, .. } => *loc,
        }
    }
}

/// Literal value
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

/// Object property
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: PropertyKey,
    pub value: Expr,
    pub kind: PropertyKind,
}

/// Property key
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    Identifier(String),
    Literal(Literal),
    Computed(Box<Expr>),
}

/// Property kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyKind {
    Init,   // Regular property
    Get,    // Getter
    Set,    // Setter
}

/// Arrow function body
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
}

/// Statement node
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement
    Expression {
        expr: Expr,
        loc: SourceLocation,
    },

    /// Block statement
    Block {
        stmts: Vec<Stmt>,
        loc: SourceLocation,
    },

    /// Variable declaration
    VarDecl {
        kind: VarKind,
        declarations: Vec<VarDeclarator>,
        loc: SourceLocation,
    },

    /// Function declaration
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        loc: SourceLocation,
    },

    /// If statement
    If {
        test: Expr,
        consequent: Box<Stmt>,
        alternate: Option<Box<Stmt>>,
        loc: SourceLocation,
    },

    /// While loop
    While {
        test: Expr,
        body: Box<Stmt>,
        loc: SourceLocation,
    },

    /// Do-while loop
    DoWhile {
        body: Box<Stmt>,
        test: Expr,
        loc: SourceLocation,
    },

    /// For loop
    For {
        init: Option<ForInit>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
        loc: SourceLocation,
    },

    /// For-in loop (iterate over object keys)
    ForIn {
        left: ForInit,
        right: Expr,
        body: Box<Stmt>,
        loc: SourceLocation,
    },

    /// For-of loop (iterate over iterable values)
    ForOf {
        left: ForInit,
        right: Expr,
        body: Box<Stmt>,
        loc: SourceLocation,
    },

    /// Break statement
    Break {
        label: Option<String>,
        loc: SourceLocation,
    },

    /// Continue statement
    Continue {
        label: Option<String>,
        loc: SourceLocation,
    },

    /// Return statement
    Return {
        argument: Option<Expr>,
        loc: SourceLocation,
    },

    /// Throw statement
    Throw {
        argument: Expr,
        loc: SourceLocation,
    },

    /// Try statement
    Try {
        block: Vec<Stmt>,
        handler: Option<CatchClause>,
        finalizer: Option<Vec<Stmt>>,
        loc: SourceLocation,
    },

    /// Switch statement
    Switch {
        discriminant: Expr,
        cases: Vec<SwitchCase>,
        loc: SourceLocation,
    },

    /// Empty statement
    Empty {
        loc: SourceLocation,
    },

    /// Labeled statement
    Labeled {
        label: String,
        body: Box<Stmt>,
        loc: SourceLocation,
    },
}

impl Stmt {
    /// Returns the source location of this statement
    pub fn location(&self) -> SourceLocation {
        match self {
            Stmt::Expression { loc, .. } |
            Stmt::Block { loc, .. } |
            Stmt::VarDecl { loc, .. } |
            Stmt::FunctionDecl { loc, .. } |
            Stmt::If { loc, .. } |
            Stmt::While { loc, .. } |
            Stmt::DoWhile { loc, .. } |
            Stmt::For { loc, .. } |
            Stmt::ForIn { loc, .. } |
            Stmt::ForOf { loc, .. } |
            Stmt::Break { loc, .. } |
            Stmt::Continue { loc, .. } |
            Stmt::Return { loc, .. } |
            Stmt::Throw { loc, .. } |
            Stmt::Try { loc, .. } |
            Stmt::Switch { loc, .. } |
            Stmt::Empty { loc, .. } |
            Stmt::Labeled { loc, .. } => *loc,
        }
    }
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

/// Variable declarator
#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclarator {
    pub name: String,
    pub init: Option<Expr>,
}

/// For loop initialization
#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    VarDecl {
        kind: VarKind,
        declarations: Vec<VarDeclarator>,
    },
    Expr(Expr),
}

/// Catch clause
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub param: Option<String>,
    pub body: Vec<Stmt>,
}

/// Switch case
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub test: Option<Expr>, // None for default case
    pub consequent: Vec<Stmt>,
}

/// Program (top-level)
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
    pub source_type: SourceType,
}

/// Source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Script,
    Module,
}

impl Program {
    /// Creates a new program
    pub fn new(body: Vec<Stmt>) -> Self {
        Program {
            body,
            source_type: SourceType::Script,
        }
    }

    /// Creates a new module
    pub fn new_module(body: Vec<Stmt>) -> Self {
        Program {
            body,
            source_type: SourceType::Module,
        }
    }
}
