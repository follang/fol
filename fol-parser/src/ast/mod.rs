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

    /// Destructuring binding declaration: var pattern = value
    DestructureDecl {
        options: Vec<VarOption>,
        is_label: bool,
        pattern: BindingPattern,
        type_hint: Option<FolType>,
        value: Box<AstNode>,
    },

    /// Function declaration: fun[options] name(params): return_type = { body }
    FunDecl {
        options: Vec<FunOption>,
        generics: Vec<Generic>,
        name: String,
        receiver_type: Option<FolType>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Procedure declaration: pro[options] name(params): return_type = { body }
    ProDecl {
        options: Vec<FunOption>,
        generics: Vec<Generic>,
        name: String,
        receiver_type: Option<FolType>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Logical declaration: log[options] name(params): return_type = { body }
    LogDecl {
        options: Vec<FunOption>,
        generics: Vec<Generic>,
        name: String,
        receiver_type: Option<FolType>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Type declaration: typ name: definition
    TypeDecl {
        options: Vec<TypeOption>,
        generics: Vec<Generic>,
        contracts: Vec<FolType>,
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
        options: Vec<DeclOption>,
        name: String,
        params: Vec<Parameter>,
        def_type: FolType,
        body: Vec<AstNode>,
    },

    /// Segment declaration: seg name: mod[...] = { body }
    SegDecl {
        options: Vec<DeclOption>,
        name: String,
        seg_type: FolType,
        body: Vec<AstNode>,
    },

    /// Implementation declaration: imp name: target_type = { body }
    ImpDecl {
        options: Vec<DeclOption>,
        generics: Vec<Generic>,
        name: String,
        target: FolType,
        body: Vec<AstNode>,
    },

    /// Standard declaration: std name: pro|blu|ext = { body }
    StdDecl {
        options: Vec<DeclOption>,
        name: String,
        kind: StandardKind,
        kind_options: Vec<DeclOption>,
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

    /// Qualified function call: a::b::call(args)
    QualifiedFunctionCall { path: QualifiedPath, args: Vec<AstNode> },

    /// Named call argument: name = value
    NamedArgument {
        name: String,
        value: Box<AstNode>,
    },

    /// Call-site unpack argument: ...value
    Unpack {
        value: Box<AstNode>,
    },

    /// Processor pipe stage: async
    AsyncStage,

    /// Processor pipe stage: await
    AwaitStage,

    /// Coroutine spawn expression: [>]expr
    Spawn {
        task: Box<AstNode>,
    },

    /// General invocation: callee(args)
    Invoke {
        callee: Box<AstNode>,
        args: Vec<AstNode>,
    },

    /// Anonymous function expression: fun (...) : T = { ... }
    AnonymousFun {
        options: Vec<FunOption>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Anonymous procedure expression: pro (...) : T = { ... }
    AnonymousPro {
        options: Vec<FunOption>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Anonymous logical expression: log (...) : bol = { ... }
    AnonymousLog {
        options: Vec<FunOption>,
        captures: Vec<String>,
        params: Vec<Parameter>,
        return_type: Option<FolType>,
        error_type: Option<FolType>,
        body: Vec<AstNode>,
        inquiries: Vec<AstNode>,
    },

    /// Method call: object.method(args)
    MethodCall {
        object: Box<AstNode>,
        method: String,
        args: Vec<AstNode>,
    },

    /// Postfix template access: object$
    TemplateCall {
        object: Box<AstNode>,
        template: String,
    },

    /// Array/Container access: container[index]
    IndexAccess {
        container: Box<AstNode>,
        index: Box<AstNode>,
    },

    /// Channel endpoint access: channel[tx] / channel[rx]
    ChannelAccess {
        channel: Box<AstNode>,
        endpoint: ChannelEndpoint,
    },

    /// Slice access: container[start:end] / container[start::end]
    SliceAccess {
        container: Box<AstNode>,
        start: Option<Box<AstNode>>,
        end: Option<Box<AstNode>>,
        reverse: bool,
    },

    /// Pattern access: container[a, b, ...]
    PatternAccess {
        container: Box<AstNode>,
        patterns: Vec<AstNode>,
    },

    /// Wildcard access pattern: *
    PatternWildcard,

    /// Capturing access pattern: pattern => Name / * => Name
    PatternCapture {
        pattern: Box<AstNode>,
        binding: String,
    },

    /// Availability access: container:[pattern] / access_expr:
    AvailabilityAccess { target: Box<AstNode> },

    /// Field access: object.field
    FieldAccess { object: Box<AstNode>, field: String },

    /// Identifier reference
    Identifier { name: String },

    /// Qualified identifier reference
    QualifiedIdentifier { path: QualifiedPath },

    /// Literal values
    Literal(Literal),

    /// Container literal: { elem1, elem2, ... }
    ContainerLiteral {
        container_type: ContainerType,
        elements: Vec<AstNode>,
    },

    /// Record/object initializer: { field = value, ... }
    RecordInit {
        fields: Vec<RecordInitField>,
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

    /// Label declaration: lab name: type = value
    LabDecl {
        options: Vec<VarOption>,
        name: String,
        type_hint: Option<FolType>,
        value: Option<Box<AstNode>>,
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

    /// Select statement: select(channel as binding) { body }
    Select {
        channel: Box<AstNode>,
        binding: Option<String>,
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

    /// Inquiry clause attached to a routine
    Inquiry {
        target: InquiryTarget,
        body: Vec<AstNode>,
    },

    /// Program root
    Program { declarations: Vec<AstNode> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedPath {
    pub segments: Vec<String>,
}

impl QualifiedPath {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    pub fn from_joined(path: &str) -> Self {
        Self {
            segments: path.split("::").map(|segment| segment.to_string()).collect(),
        }
    }

    pub fn is_qualified(&self) -> bool {
        self.segments.len() > 1
    }

    pub fn joined(&self) -> String {
        self.segments.join("::")
    }
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
    Channel {
        element_type: Box<FolType>,
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
    Union {
        types: Vec<FolType>,
    }, // uni[T1, T2, ...]
    Never,
    Any,
    Pointer {
        target: Box<FolType>,
    },
    Error {
        inner: Option<Box<FolType>>,
    },
    Limited {
        base: Box<FolType>,
        limits: Vec<AstNode>,
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
    Test {
        name: Option<String>,
        access: Vec<String>,
    },
    Url {
        name: String,
    },
    Location {
        name: String,
    },
    Standard {
        name: String,
    },

    // User-defined type reference
    Named {
        name: String,
    },
    QualifiedNamed {
        path: QualifiedPath,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InquiryTarget {
    SelfValue,
    ThisValue,
    Named(String),
    Quoted(String),
    Qualified(QualifiedPath),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindingPattern {
    Name(String),
    Rest(String),
    Sequence(Vec<BindingPattern>),
}

impl BindingPattern {
    pub fn is_destructuring(&self) -> bool {
        match self {
            BindingPattern::Name(_) => false,
            BindingPattern::Rest(_) => true,
            BindingPattern::Sequence(parts) => {
                parts.len() != 1 || parts.iter().any(BindingPattern::is_destructuring)
            }
        }
    }
}

impl InquiryTarget {
    pub fn duplicate_key(&self) -> String {
        match self {
            InquiryTarget::SelfValue => "self".to_string(),
            InquiryTarget::ThisValue => "this".to_string(),
            InquiryTarget::Named(name) | InquiryTarget::Quoted(name) => name.clone(),
            InquiryTarget::Qualified(path) => path.joined(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            InquiryTarget::SelfValue => "self".to_string(),
            InquiryTarget::ThisValue => "this".to_string(),
            InquiryTarget::Named(name) => name.clone(),
            InquiryTarget::Quoted(name) => format!("\"{}\"", name),
            InquiryTarget::Qualified(path) => path.joined(),
        }
    }
}

impl FolType {
    pub fn named_text(&self) -> Option<String> {
        match self {
            FolType::Named { name } => Some(name.clone()),
            FolType::QualifiedNamed { path } => Some(path.joined()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StandardKind {
    Protocol,
    Blueprint,
    Extended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclOption {
    Export,
    Hidden,
    Normal,
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
    pub is_mutex: bool,
    pub default: Option<AstNode>,
}

/// Generic type parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Generic {
    pub name: String,
    pub constraints: Vec<FolType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordFieldMeta {
    pub default: Option<AstNode>,
    pub options: Vec<VarOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryVariantMeta {
    pub default: Option<AstNode>,
    pub options: Vec<VarOption>,
}

/// Type definitions for structs/records/entries
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition {
    Record {
        fields: HashMap<String, FolType>,
        field_meta: HashMap<String, RecordFieldMeta>,
        members: Vec<AstNode>,
    },
    Entry {
        variants: HashMap<String, Option<FolType>>,
        variant_meta: HashMap<String, EntryVariantMeta>,
        members: Vec<AstNode>,
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
    Nil,
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
    Pipe,
    PipeOr,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
    Ref,
    Deref,
    Unwrap,
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
    Export,    // exp or +
    Set,       // set
    Get,       // get
    Nothing,   // nothing
    Extension, // ext
}

/// Use declaration options
#[derive(Debug, Clone, PartialEq)]
pub enum UseOption {
    Export,
    Hidden,
    Normal,
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
pub enum ChannelEndpoint {
    Tx,
    Rx,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBinding {
    pub name: String,
    pub type_hint: Option<FolType>,
    pub iterable: AstNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordInitField {
    pub name: String,
    pub value: AstNode,
}

impl AstNode {
    /// Return the direct body nodes for routine-like AST nodes.
    pub fn routine_body(&self) -> Option<&[AstNode]> {
        match self {
            AstNode::FunDecl { body, .. }
            | AstNode::ProDecl { body, .. }
            | AstNode::LogDecl { body, .. }
            | AstNode::AnonymousFun { body, .. }
            | AstNode::AnonymousPro { body, .. }
            | AstNode::AnonymousLog { body, .. } => Some(body.as_slice()),
            _ => None,
        }
    }

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
            AstNode::Literal(Literal::Nil) => Some(FolType::None),

            AstNode::VarDecl { type_hint, .. }
            | AstNode::LabDecl { type_hint, .. }
            | AstNode::DestructureDecl { type_hint, .. } => {
                type_hint.clone()
            }
            AstNode::FunDecl { return_type, .. } => return_type.clone(),
            AstNode::ProDecl { return_type, .. } => return_type.clone(),
            AstNode::LogDecl { return_type, .. } => {
                return_type.clone().or(Some(FolType::Bool))
            }
            AstNode::DefDecl { def_type, .. } => Some(def_type.clone()),
            AstNode::SegDecl { seg_type, .. } => Some(seg_type.clone()),
            AstNode::ImpDecl { target, .. } => Some(target.clone()),
            AstNode::StdDecl { .. } => None,

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
                UnaryOperator::Unwrap => {
                    if let Some(FolType::Optional { inner }) = operand.get_type() {
                        Some(*inner)
                    } else {
                        operand.get_type()
                    }
                }
            },
            AstNode::Invoke { callee, .. } => {
                if let Some(FolType::Function { return_type, .. }) = callee.get_type() {
                    Some(*return_type)
                } else {
                    None
                }
            }
            AstNode::NamedArgument { value, .. } => value.get_type(),
            AstNode::Unpack { value } => value.get_type(),
            AstNode::AsyncStage | AstNode::AwaitStage => None,
            AstNode::Spawn { task } => task.get_type(),
            AstNode::AnonymousFun {
                params,
                return_type,
                ..
            } => Some(FolType::Function {
                params: params.iter().map(|param| param.param_type.clone()).collect(),
                return_type: Box::new(return_type.clone().unwrap_or(FolType::Any)),
            }),
            AstNode::AnonymousPro {
                params,
                return_type,
                ..
            } => Some(FolType::Function {
                params: params.iter().map(|param| param.param_type.clone()).collect(),
                return_type: Box::new(return_type.clone().unwrap_or(FolType::Any)),
            }),
            AstNode::AnonymousLog {
                params,
                return_type,
                ..
            } => Some(FolType::Function {
                params: params.iter().map(|param| param.param_type.clone()).collect(),
                return_type: Box::new(return_type.clone().unwrap_or(FolType::Bool)),
            }),
            AstNode::AvailabilityAccess { .. } => Some(FolType::Bool),
            AstNode::Inquiry { .. } => None,
            AstNode::PatternWildcard => None,
            AstNode::PatternCapture { pattern, .. } => pattern.get_type(),
            AstNode::RecordInit { .. } => None,
            AstNode::TemplateCall { .. } => None,

            _ => None,
        }
    }

    /// Get all child nodes for tree traversal
    pub fn children(&self) -> Vec<&AstNode> {
        match self {
            AstNode::VarDecl { value, .. } | AstNode::LabDecl { value, .. } => {
                value.as_ref().map(|v| vec![v.as_ref()]).unwrap_or_default()
            }
            AstNode::DestructureDecl { value, .. } => vec![value.as_ref()],
            AstNode::FunDecl {
                body, inquiries, ..
            }
            | AstNode::ProDecl {
                body, inquiries, ..
            }
            | AstNode::LogDecl {
                body, inquiries, ..
            } => {
                let mut children: Vec<&AstNode> = body.iter().collect();
                children.extend(inquiries.iter());
                children
            }
            AstNode::DefDecl { body, .. }
            | AstNode::SegDecl { body, .. }
            | AstNode::ImpDecl { body, .. }
            | AstNode::StdDecl { body, .. } => body.iter().collect(),
            AstNode::Inquiry { body, .. } => body.iter().collect(),
            AstNode::BinaryOp { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            AstNode::UnaryOp { operand, .. } => {
                vec![operand.as_ref()]
            }
            AstNode::NamedArgument { value, .. } => {
                vec![value.as_ref()]
            }
            AstNode::Unpack { value } => {
                vec![value.as_ref()]
            }
            AstNode::AsyncStage | AstNode::AwaitStage => vec![],
            AstNode::Spawn { task } => vec![task.as_ref()],
            AstNode::FunctionCall { args, .. } | AstNode::QualifiedFunctionCall { args, .. } => {
                args.iter().collect()
            }
            AstNode::MethodCall { object, args, .. } => {
                let mut children = vec![object.as_ref()];
                children.extend(args.iter());
                children
            }
            AstNode::Invoke { callee, args } => {
                let mut children = vec![callee.as_ref()];
                children.extend(args.iter());
                children
            }
            AstNode::TemplateCall { object, .. } => vec![object.as_ref()],
            AstNode::AnonymousFun {
                body, inquiries, ..
            }
            | AstNode::AnonymousPro {
                body, inquiries, ..
            }
            | AstNode::AnonymousLog {
                body, inquiries, ..
            } => {
                let mut children: Vec<&AstNode> = body.iter().collect();
                children.extend(inquiries.iter());
                children
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
            AstNode::Select { channel, body, .. } => {
                let mut children = vec![channel.as_ref()];
                children.extend(body.iter());
                children
            }
            AstNode::Block { statements } => statements.iter().collect(),
            AstNode::Program { declarations } => declarations.iter().collect(),
            AstNode::ContainerLiteral { elements, .. } => elements.iter().collect(),
            AstNode::RecordInit { fields } => fields.iter().map(|field| &field.value).collect(),
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
            AstNode::ChannelAccess { channel, .. } => vec![channel.as_ref()],
            AstNode::SliceAccess {
                container,
                start,
                end,
                ..
            } => {
                let mut children = vec![container.as_ref()];
                if let Some(start) = start {
                    children.push(start.as_ref());
                }
                if let Some(end) = end {
                    children.push(end.as_ref());
                }
                children
            }
            AstNode::PatternAccess {
                container,
                patterns,
            } => {
                let mut children = vec![container.as_ref()];
                children.extend(patterns.iter());
                children
            }
            AstNode::PatternCapture { pattern, .. } => vec![pattern.as_ref()],
            AstNode::AvailabilityAccess { target } => {
                vec![target.as_ref()]
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
