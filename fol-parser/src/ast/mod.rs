// Proper AST Design for FOL Language
// Based on FOL language specification from the book

use std::collections::HashMap;

/// Core AST node types for FOL language
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    // ==== DECLARATIONS ====
    /// Variable declaration: var[options] name: type = value
    VarDecl {
        options: Vec<VarOption>,
        name: String,
        type_hint: Option<FolType>,
        value: Option<Box<AstNode>>,
    },

    /// Function declaration: fun[options] name(params): return_type = { body }
    FunDecl {
        options: Vec<FunOption>,
        generics: Vec<Generic>,
        name: String,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
    },

    /// Procedure declaration: pro[options] name(params): return_type = { body }
    ProDecl {
        options: Vec<FunOption>,
        generics: Vec<Generic>,
        name: String,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
    },

    /// Type declaration: typ name: definition
    TypeDecl {
        options: Vec<TypeOption>,
        generics: Vec<Generic>,
        name: String,
        type_def: TypeDefinition,
    },

    /// Use declaration: use[options] name: type = { path }
    UseDecl {
        options: Vec<UseOption>,
        name: String,
        path_type: FolType,
        path: String,
    },

    /// Alias declaration: ali name: target_type
    AliasDecl { name: String, target: FolType },

    /// Definition declaration: def name: mod[...] = { body } / def name: blk[...] = { body }
    DefDecl {
        name: String,
        def_type: FolType,
        body: Vec<AstNode>,
    },

    /// Implementation declaration: imp name: target_type = { body }
    ImpDecl {
        generics: Vec<Generic>,
        name: String,
        target: FolType,
        body: Vec<AstNode>,
    },

    // ==== EXPRESSIONS ====
    /// Binary operation: (left op right)
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    /// Unary operation: (op operand)
    UnaryOp {
        op: UnaryOperator,
        operand: Box<AstNode>,
    },

    /// Function call: function_name(args)
    FunctionCall { name: String, args: Vec<AstNode> },

    /// Method call: object.method(args)
    MethodCall {
        object: Box<AstNode>,
        method: String,
        args: Vec<AstNode>,
    },

    /// Array/Container access: container[index]
    IndexAccess {
        container: Box<AstNode>,
        index: Box<AstNode>,
    },

    /// Field access: object.field
    FieldAccess { object: Box<AstNode>, field: String },

    /// Identifier reference
    Identifier { name: String },

    /// Literal values
    Literal(Literal),

    /// Container literal: { elem1, elem2, ... }
    ContainerLiteral {
        container_type: ContainerType,
        elements: Vec<AstNode>,
    },

    /// Rolling/list-comprehension expression: { expr for x in iterable if cond }
    Rolling {
        expr: Box<AstNode>,
        bindings: Vec<RollingBinding>,
        condition: Option<Box<AstNode>>,
    },

    /// Range expression: {start..end}
    Range {
        start: Option<Box<AstNode>>,
        end: Option<Box<AstNode>>,
        inclusive: bool,
    },

    // ==== STATEMENTS ====
    /// Assignment: target = value
    Assignment {
        target: Box<AstNode>,
        value: Box<AstNode>,
    },

    /// When statement (FOL's if/match): when(expr) { case(condition){} * {} }
    When {
        expr: Box<AstNode>,
        cases: Vec<WhenCase>,
        default: Option<Vec<AstNode>>,
    },

    /// Loop statement: loop(condition) { body }
    Loop {
        condition: Box<LoopCondition>,
        body: Vec<AstNode>,
    },

    /// Return statement: return value
    Return { value: Option<Box<AstNode>> },

    /// Break statement
    Break,

    /// Yield statement: yield value
    Yield { value: Box<AstNode> },

    /// Block: { statements }
    Block { statements: Vec<AstNode> },

    /// Program root
    Program { declarations: Vec<AstNode> },
}

/// FOL Type system
#[derive(Debug, Clone, PartialEq)]
pub enum FolType {
    // Ordinal types
    Int {
        size: Option<IntSize>,
        signed: bool,
    },
    Float {
        size: Option<FloatSize>,
    },
    Char {
        encoding: CharEncoding,
    },
    Bool,

    // Container types
    Array {
        element_type: Box<FolType>,
        size: Option<usize>,
    },
    Vector {
        element_type: Box<FolType>,
    },
    Sequence {
        element_type: Box<FolType>,
    },
    Matrix {
        element_type: Box<FolType>,
        dimensions: Vec<usize>,
    },
    Set {
        types: Vec<FolType>,
    }, // Tuple-like heterogeneous set
    Map {
        key_type: Box<FolType>,
        value_type: Box<FolType>,
    },

    // Complex types
    Record {
        fields: HashMap<String, FolType>,
    },
    Entry {
        variants: HashMap<String, Option<FolType>>,
    }, // Enum-like

    // Special types
    Optional {
        inner: Box<FolType>,
    }, // opt[T]
    Multiple {
        types: Vec<FolType>,
    }, // mul[T1, T2, ...]
    Any,
    Pointer {
        target: Box<FolType>,
    },
    Error {
        inner: Option<Box<FolType>>,
    },
    None,

    // Function types
    Function {
        params: Vec<FolType>,
        return_type: Box<FolType>,
    },

    // Generic and module types
    Generic {
        name: String,
        constraints: Vec<FolType>,
    },
    Module {
        name: String,
    },
    Block {
        name: String,
    },

    // User-defined type reference
    Named {
        name: String,
    },
}

/// Integer sizes
#[derive(Debug, Clone, PartialEq)]
pub enum IntSize {
    I8,
    I16,
    I32,
    I64,
    I128,
    Arch,
}

/// Float sizes  
#[derive(Debug, Clone, PartialEq)]
pub enum FloatSize {
    F32,
    F64,
    Arch,
}

/// Character encodings
#[derive(Debug, Clone, PartialEq)]
pub enum CharEncoding {
    Utf8,
    Utf16,
    Utf32,
}

/// Container type for literals
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    Array,
    Vector,
    Sequence,
    Set,
    Map,
}

/// Function/Procedure parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: FolType,
    pub is_borrowable: bool, // ALL_CAPS names are borrowable
    pub default: Option<AstNode>,
}

/// Generic type parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Generic {
    pub name: String,
    pub constraints: Vec<FolType>,
}

/// Type definitions for structs/records/entries
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition {
    Record {
        fields: HashMap<String, FolType>,
    },
    Entry {
        variants: HashMap<String, Option<FolType>>,
    },
    Alias {
        target: FolType,
    },
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Character(char),
    Boolean(bool),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
    Xor,

    // Other
    In,
    Has,
    Is,
    As,
    Cast,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
    Ref,
    Deref,
}

/// Variable declaration options
#[derive(Debug, Clone, PartialEq)]
pub enum VarOption {
    Mutable,   // mut or ~
    Immutable, // imu (default)
    Static,    // sta or !
    Reactive,  // rac or ?
    Export,    // exp or +
    Normal,    // nor (default)
    Hidden,    // hid or -
    New,       // allocate on heap
    Borrowing, // bor - borrowing a value
}

/// Function/Procedure options
#[derive(Debug, Clone, PartialEq)]
pub enum FunOption {
    Export,   // exp or +
    Mutable,  // mut
    Iterator, // itr
}

/// Type declaration options
#[derive(Debug, Clone, PartialEq)]
pub enum TypeOption {
    Export,  // exp or +
    Set,     // set
    Get,     // get
    Nothing, // nothing
}

/// Use declaration options
#[derive(Debug, Clone, PartialEq)]
pub enum UseOption {
    // Add specific use options as needed
}

/// When statement cases
#[derive(Debug, Clone, PartialEq)]
pub enum WhenCase {
    /// case(condition) { body }
    Case {
        condition: AstNode,
        body: Vec<AstNode>,
    },
    /// is(value) { body } - for value matching
    Is { value: AstNode, body: Vec<AstNode> },
    /// in(range/set) { body } - for range/set matching
    In { range: AstNode, body: Vec<AstNode> },
    /// has(member) { body } - for containment checking
    Has { member: AstNode, body: Vec<AstNode> },
    /// of(type) { body } - for type matching
    Of {
        type_match: FolType,
        body: Vec<AstNode>,
    },
    /// on(channel) { body } - for channel matching
    On {
        channel: AstNode,
        body: Vec<AstNode>,
    },
}

/// Loop condition types
#[derive(Debug, Clone, PartialEq)]
pub enum LoopCondition {
    /// loop(condition) - while-like loop
    Condition(Box<AstNode>),
    /// loop(var in iterable) - for-like loop
    Iteration {
        var: String,
        type_hint: Option<FolType>,
        iterable: Box<AstNode>,
        condition: Option<Box<AstNode>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBinding {
    pub name: String,
    pub type_hint: Option<FolType>,
    pub iterable: AstNode,
}

impl AstNode {
    /// Get the type of this AST node (for type inference)
    pub fn get_type(&self) -> Option<FolType> {
        match self {
            AstNode::Literal(Literal::Integer(_)) => Some(FolType::Int {
                size: None,
                signed: true,
            }),
            AstNode::Literal(Literal::Float(_)) => Some(FolType::Float { size: None }),
            AstNode::Literal(Literal::String(_)) => Some(FolType::Named {
                name: "str".to_string(),
            }),
            AstNode::Literal(Literal::Character(_)) => Some(FolType::Char {
                encoding: CharEncoding::Utf8,
            }),
            AstNode::Literal(Literal::Boolean(_)) => Some(FolType::Bool),

            AstNode::VarDecl { type_hint, .. } => type_hint.clone(),
            AstNode::FunDecl { return_type, .. } => return_type.clone(),
            AstNode::ProDecl { return_type, .. } => return_type.clone(),
            AstNode::DefDecl { def_type, .. } => Some(def_type.clone()),
            AstNode::ImpDecl { target, .. } => Some(target.clone()),

            AstNode::BinaryOp { op, left, right } => {
                // Type inference for binary operations
                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Mod
                    | BinaryOperator::Pow => {
                        // Arithmetic operations - return type of operands (with promotion rules)
                        left.get_type().or_else(|| right.get_type())
                    }
                    BinaryOperator::Eq
                    | BinaryOperator::Ne
                    | BinaryOperator::Lt
                    | BinaryOperator::Le
                    | BinaryOperator::Gt
                    | BinaryOperator::Ge
                    | BinaryOperator::In
                    | BinaryOperator::Has
                    | BinaryOperator::Is => {
                        // Comparison operations always return bool
                        Some(FolType::Bool)
                    }
                    BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
                        // Logical operations return bool
                        Some(FolType::Bool)
                    }
                    _ => None,
                }
            }

            AstNode::UnaryOp { op, operand } => match op {
                UnaryOperator::Neg => operand.get_type(),
                UnaryOperator::Not => Some(FolType::Bool),
                UnaryOperator::Ref => operand.get_type().map(|t| FolType::Pointer {
                    target: Box::new(t),
                }),
                UnaryOperator::Deref => {
                    if let Some(FolType::Pointer { target }) = operand.get_type() {
                        Some(*target)
                    } else {
                        None
                    }
                }
            },

            _ => None,
        }
    }

    /// Get all child nodes for tree traversal
    pub fn children(&self) -> Vec<&AstNode> {
        match self {
            AstNode::VarDecl { value, .. } => {
                value.as_ref().map(|v| vec![v.as_ref()]).unwrap_or_default()
            }
            AstNode::FunDecl { body, .. }
            | AstNode::ProDecl { body, .. }
            | AstNode::DefDecl { body, .. }
            | AstNode::ImpDecl { body, .. } => body.iter().collect(),
            AstNode::BinaryOp { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AstNode::UnaryOp { operand, .. } => {
                vec![operand.as_ref()]
            }
            AstNode::FunctionCall { args, .. } | AstNode::MethodCall { args, .. } => {
                args.iter().collect()
            }
            AstNode::Assignment { target, value } => {
                vec![target.as_ref(), value.as_ref()]
            }
            AstNode::When {
                expr,
                cases,
                default,
            } => {
                let mut children = vec![expr.as_ref()];
                for case in cases {
                    match case {
                        WhenCase::Case { condition, body }
                        | WhenCase::Is {
                            value: condition,
                            body,
                        }
                        | WhenCase::In {
                            range: condition,
                            body,
                        }
                        | WhenCase::Has {
                            member: condition,
                            body,
                        }
                        | WhenCase::On {
                            channel: condition,
                            body,
                        } => {
                            children.push(condition);
                            children.extend(body.iter());
                        }
                        WhenCase::Of { body, .. } => {
                            children.extend(body.iter());
                        }
                    }
                }
                if let Some(default_body) = default {
                    children.extend(default_body.iter());
                }
                children
            }
            AstNode::Loop { condition, body } => {
                let mut children: Vec<&AstNode> = body.iter().collect();
                match condition.as_ref() {
                    LoopCondition::Condition(cond) => children.push(cond.as_ref()),
                    LoopCondition::Iteration {
                        iterable,
                        condition: iter_cond,
                        ..
                    } => {
                        children.push(iterable.as_ref());
                        if let Some(cond) = iter_cond {
                            children.push(cond.as_ref());
                        }
                    }
                }
                children
            }
            AstNode::Block { statements } => statements.iter().collect(),
            AstNode::Program { declarations } => declarations.iter().collect(),
            AstNode::ContainerLiteral { elements, .. } => elements.iter().collect(),
            AstNode::Rolling {
                expr,
                bindings,
                condition,
            } => {
                let mut children = vec![expr.as_ref()];
                children.extend(bindings.iter().map(|binding| &binding.iterable));
                if let Some(cond) = condition {
                    children.push(cond.as_ref());
                }
                children
            }
            AstNode::IndexAccess { container, index } => {
                vec![container.as_ref(), index.as_ref()]
            }
            AstNode::FieldAccess { object, .. } => {
                vec![object.as_ref()]
            }
            AstNode::Return { value } => {
                value.as_ref().map(|v| vec![v.as_ref()]).unwrap_or_default()
            }
            AstNode::Yield { value } => {
                vec![value.as_ref()]
            }
            AstNode::Range { start, end, .. } => {
                let mut children = Vec::new();
                if let Some(s) = start {
                    children.push(s.as_ref());
                }
                if let Some(e) = end {
                    children.push(e.as_ref());
                }
                children
            }
            _ => vec![],
        }
    }

    /// Accept a visitor for the visitor pattern
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit(self);
    }
}

/// Visitor pattern for AST traversal and analysis
pub trait AstVisitor {
    fn visit(&mut self, node: &AstNode);

    fn visit_var_decl(
        &mut self,
        _options: &[VarOption],
        _name: &str,
        _type_hint: &Option<FolType>,
        _value: &Option<Box<AstNode>>,
    ) {
    }
    fn visit_fun_decl(
        &mut self,
        _options: &[FunOption],
        _name: &str,
        _params: &[Parameter],
        _return_type: &Option<FolType>,
        _body: &[AstNode],
    ) {
    }
    fn visit_pro_decl(
        &mut self,
        _options: &[FunOption],
        _name: &str,
        _params: &[Parameter],
        _return_type: &Option<FolType>,
        _body: &[AstNode],
    ) {
    }
    fn visit_type_decl(
        &mut self,
        _options: &[TypeOption],
        _name: &str,
        _type_def: &TypeDefinition,
    ) {
    }
    fn visit_binary_op(&mut self, _op: &BinaryOperator, _left: &AstNode, _right: &AstNode) {}
    fn visit_unary_op(&mut self, _op: &UnaryOperator, _operand: &AstNode) {}
    fn visit_function_call(&mut self, _name: &str, _args: &[AstNode]) {}
    fn visit_identifier(&mut self, _name: &str) {}
    fn visit_literal(&mut self, _literal: &Literal) {}
    fn visit_assignment(&mut self, _target: &AstNode, _value: &AstNode) {}
    fn visit_when(
        &mut self,
        _expr: &AstNode,
        _cases: &[WhenCase],
        _default: &Option<Vec<AstNode>>,
    ) {
    }
    fn visit_loop(&mut self, _condition: &LoopCondition, _body: &[AstNode]) {}
    fn visit_block(&mut self, _statements: &[AstNode]) {}
    fn visit_program(&mut self, _declarations: &[AstNode]) {}
}

// Re-export modules
pub mod parser;
pub use parser::{AstParser, ParseError};
