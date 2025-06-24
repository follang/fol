# FOL Programming Language - AST Completion Planning

## Current AST Status

### What We Have ✅
- **Basic AST structure**: Core `AstNode` enum with Program, Declaration, Expression, Statement variants
- **Declaration parsing**: Complete implementation for `use`, `ali`, `imp`, `seg`, `lab`, `def` declarations
- **Basic expression parsing**: Literals, identifiers, simple binary/unary operations
- **Token integration**: Full lexer-to-parser token flow
- **Error handling**: Basic parse error reporting and recovery

### What's Missing ❌
- **Complete expression parsing**: Complex expressions, precedence handling, function calls
- **Statement parsing**: Control flow, variable declarations, assignments
- **Type system integration**: Type annotations, type checking, inference
- **Semantic analysis**: Symbol tables, scope resolution, type validation
- **Advanced AST features**: Pattern matching, generics, advanced control flow

## Phase 1: Complete Expression Parsing

### Priority: HIGH 🔥

#### 1.1 Operator Precedence and Associativity
**Location**: `fol-parser/src/parser/expression.rs`

```rust
// Need to implement precedence climbing or Pratt parsing
enum Precedence {
    None,
    Assignment,    // =
    Or,           // ||
    And,          // &&
    Equality,     // == !=
    Comparison,   // > >= < <=
    Term,         // + -
    Factor,       // * /
    Unary,        // ! -
    Call,         // . ()
    Primary,
}
```

**Tasks**:
- [ ] Implement precedence table for all operators
- [ ] Add Pratt parser or precedence climbing algorithm
- [ ] Handle left/right associativity correctly
- [ ] Add comprehensive operator tests

#### 1.2 Function Call Expressions
**Current Gap**: No function call parsing

```rust
// Need to parse: func(arg1, arg2, arg3)
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub location: Location,
}
```

**Tasks**:
- [ ] Parse function call syntax `identifier(args)`
- [ ] Handle method calls `object.method(args)`
- [ ] Support nested function calls
- [ ] Parse argument lists with proper comma handling
- [ ] Add variadic argument support

#### 1.3 Complex Expression Types
**Missing expressions**:

```rust
pub enum ExpressionKind {
    // ✅ Already implemented
    Literal(LiteralExpression),
    Identifier(IdentifierExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    
    // ❌ Need to implement
    Call(CallExpression),           // func(args)
    Index(IndexExpression),         // arr[index]
    Member(MemberExpression),       // obj.field
    Array(ArrayExpression),         // [1, 2, 3]
    Tuple(TupleExpression),         // (a, b, c)
    Lambda(LambdaExpression),       // |x| x + 1
    Conditional(ConditionalExpression), // condition ? true_expr : false_expr
    Block(BlockExpression),         // { statements; expr }
}
```

**Tasks**:
- [ ] Array/list literal parsing: `[1, 2, 3, 4]`
- [ ] Tuple literal parsing: `(a, b, c)`
- [ ] Index expression parsing: `array[index]`
- [ ] Member access parsing: `object.field`
- [ ] Lambda/closure parsing: `|param| body`
- [ ] Conditional expressions: `condition ? true_val : false_val`
- [ ] Block expressions with return values

#### 1.4 Type Annotations in Expressions
**Current Gap**: No type annotation parsing

```rust
// Need to parse: variable: Type = value
pub struct TypedExpression {
    pub expression: Box<Expression>,
    pub type_annotation: Option<TypeExpression>,
    pub location: Location,
}
```

## Phase 2: Complete Statement Parsing

### Priority: HIGH 🔥

#### 2.1 Variable Declaration Statements
**Current Gap**: Basic parsing exists but needs enhancement

```rust
pub enum StatementKind {
    // ✅ Basic implementation exists
    Expression(ExpressionStatement),
    
    // ❌ Need to implement/enhance
    Let(LetStatement),           // let x = value;
    Var(VarStatement),           // var x = value;
    Assignment(AssignmentStatement), // x = value;
    Compound(CompoundStatement), // x += value;
}
```

**Tasks**:
- [ ] Parse `let` declarations with type inference: `let x = 42;`
- [ ] Parse `let` with explicit types: `let x: Int = 42;`
- [ ] Parse `var` mutable declarations: `var x = 42;`
- [ ] Parse destructuring assignments: `let (a, b) = tuple;`
- [ ] Parse pattern matching in declarations: `let Some(x) = option;`

#### 2.2 Control Flow Statements
**Current Gap**: No control flow parsing

```rust
pub enum StatementKind {
    // ❌ Need to implement
    If(IfStatement),           // if condition { } else { }
    While(WhileStatement),     // while condition { }
    For(ForStatement),         // for item in iterator { }
    Match(MatchStatement),     // match value { pattern => expr }
    Return(ReturnStatement),   // return expr;
    Break(BreakStatement),     // break;
    Continue(ContinueStatement), // continue;
}
```

**Tasks**:
- [ ] Parse `if/else` statements: `if condition { } else { }`
- [ ] Parse `while` loops: `while condition { statements }`
- [ ] Parse `for` loops: `for item in collection { statements }`
- [ ] Parse `match` statements with pattern matching
- [ ] Parse `return` statements: `return expression;`
- [ ] Parse `break` and `continue` statements
- [ ] Handle nested control flow correctly

#### 2.3 Block Statements and Scoping
**Current Gap**: Basic blocks exist but need scoping

```rust
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub scope: ScopeId, // Need to add scoping
    pub location: Location,
}
```

**Tasks**:
- [ ] Parse block statements: `{ statement1; statement2; }`
- [ ] Handle block scoping rules
- [ ] Parse blocks with return expressions: `{ statements; final_expr }`
- [ ] Support nested blocks with proper scoping

## Phase 3: Type System Integration

### Priority: MEDIUM 🟡

#### 3.1 Type Expression Parsing
**Current Gap**: No type system parsing

```rust
pub enum TypeExpression {
    Named(String),              // Int, String, Bool
    Generic(String, Vec<TypeExpression>), // Vec<Int>, Map<String, Int>
    Function(FunctionType),     // (Int, String) -> Bool
    Tuple(Vec<TypeExpression>), // (Int, String, Bool)
    Array(Box<TypeExpression>), // [Int]
    Optional(Box<TypeExpression>), // Int?
}
```

**Tasks**:
- [ ] Parse basic types: `Int`, `String`, `Bool`, `Float`
- [ ] Parse generic types: `Vec<T>`, `Map<K, V>`
- [ ] Parse function types: `(Int, String) -> Bool`
- [ ] Parse tuple types: `(Int, String, Bool)`
- [ ] Parse array types: `[Int]`, `[String]`
- [ ] Parse optional types: `Int?`, `String?`

#### 3.2 Generic Type Parameters
**Current Gap**: No generic parsing

```rust
pub struct GenericDeclaration {
    pub name: String,
    pub bounds: Vec<TypeExpression>, // T: Clone + Debug
    pub default: Option<TypeExpression>, // T = Int
}
```

**Tasks**:
- [ ] Parse generic function definitions: `def func<T>(param: T) -> T`
- [ ] Parse type constraints: `T: Clone + Debug`
- [ ] Parse default generic parameters: `T = Int`
- [ ] Handle generic type inference

#### 3.3 Type Annotations Integration
**Current Gap**: Type annotations not integrated into AST

**Tasks**:
- [ ] Add type annotations to variable declarations
- [ ] Add type annotations to function parameters
- [ ] Add return type annotations to functions
- [ ] Support type inference markers

## Phase 4: Advanced AST Features

### Priority: MEDIUM 🟡

#### 4.1 Pattern Matching
**Current Gap**: No pattern matching support

```rust
pub enum Pattern {
    Wildcard,                   // _
    Identifier(String),         // x
    Literal(LiteralExpression), // 42, "hello"
    Tuple(Vec<Pattern>),        // (a, b, c)
    Array(Vec<Pattern>),        // [first, second, rest...]
    Struct(String, Vec<(String, Pattern)>), // Point { x, y }
    Enum(String, Vec<Pattern>), // Some(value)
}
```

**Tasks**:
- [ ] Parse wildcard patterns: `_`
- [ ] Parse identifier patterns: `x`, `value`
- [ ] Parse literal patterns: `42`, `"hello"`
- [ ] Parse tuple patterns: `(a, b, c)`
- [ ] Parse array patterns: `[first, ...rest]`
- [ ] Parse struct patterns: `Point { x, y }`
- [ ] Parse enum patterns: `Some(value)`, `None`

#### 4.2 Advanced Control Flow
**Current Gap**: Advanced control flow missing

```rust
pub enum StatementKind {
    // ❌ Advanced features needed
    Try(TryStatement),         // try { } catch { }
    Defer(DeferStatement),     // defer cleanup();
    Async(AsyncStatement),     // async { }
    Await(AwaitExpression),    // await future
}
```

**Tasks**:
- [ ] Parse `try/catch` error handling
- [ ] Parse `defer` statements for cleanup
- [ ] Parse `async/await` for asynchronous code
- [ ] Parse `yield` for generators

#### 4.3 Module and Import System Enhancement
**Current Gap**: Basic `use` parsing needs enhancement

```rust
pub enum ImportKind {
    Simple(String),            // use module
    Qualified(String, String), // use module::item
    Wildcard(String),          // use module::*
    Aliased(String, String),   // use module::item as alias
    Multiple(String, Vec<String>), // use module::{item1, item2}
}
```

**Tasks**:
- [ ] Parse qualified imports: `use std::collections::HashMap`
- [ ] Parse wildcard imports: `use module::*`
- [ ] Parse aliased imports: `use long::module::name as short`
- [ ] Parse multiple imports: `use module::{item1, item2, item3}`
- [ ] Parse relative imports: `use ./local_module`

## Phase 5: Semantic Analysis Integration

### Priority: LOW 🟢

#### 5.1 Symbol Table Integration
**Current Gap**: No semantic analysis

```rust
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
}

pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}
```

**Tasks**:
- [ ] Build symbol tables during parsing
- [ ] Track variable declarations and usage
- [ ] Handle scope resolution
- [ ] Detect undeclared variables
- [ ] Handle shadowing rules

#### 5.2 Type Checking Integration
**Current Gap**: No type validation

**Tasks**:
- [ ] Type check expressions during parsing
- [ ] Validate function call arguments
- [ ] Check assignment compatibility
- [ ] Infer types where possible
- [ ] Report type errors with precise locations

#### 5.3 Semantic Validation
**Current Gap**: No semantic checks

**Tasks**:
- [ ] Validate control flow (unreachable code)
- [ ] Check for unused variables
- [ ] Validate function return paths
- [ ] Check for infinite loops
- [ ] Validate pattern matching exhaustiveness

## Implementation Strategy

### Phase Priority Order
1. **Phase 1** (Expression Parsing) - Essential for basic functionality
2. **Phase 2** (Statement Parsing) - Required for complete programs
3. **Phase 3** (Type System) - Needed for type safety
4. **Phase 4** (Advanced Features) - Language completeness
5. **Phase 5** (Semantic Analysis) - Code correctness

### Testing Strategy
For each phase:
- [ ] Unit tests for each new AST node type
- [ ] Integration tests for parser combinations
- [ ] Error handling tests for malformed input
- [ ] Performance tests for complex parsing scenarios

### Code Organization
```
fol-parser/src/
├── parser/
│   ├── expression.rs      # Phase 1 implementation
│   ├── statement.rs       # Phase 2 implementation
│   ├── types.rs          # Phase 3 implementation
│   ├── patterns.rs       # Phase 4 implementation
│   └── semantic.rs       # Phase 5 implementation
├── ast/
│   ├── nodes.rs          # AST node definitions
│   ├── visitor.rs        # AST visitor pattern
│   └── builder.rs        # AST builder utilities
└── tests/
    ├── expressions.rs    # Expression parsing tests
    ├── statements.rs     # Statement parsing tests
    └── integration.rs    # Full parser tests
```

## Success Criteria

### Complete AST Parser Should:
- [ ] Parse all FOL language constructs correctly
- [ ] Generate well-formed AST for valid programs
- [ ] Provide precise error messages for invalid programs
- [ ] Handle edge cases and malformed input gracefully
- [ ] Maintain performance for large source files
- [ ] Integrate seamlessly with lexer and diagnostic systems

### Deliverables
1. **Complete AST node hierarchy** covering all language constructs
2. **Recursive descent parser** with proper error recovery
3. **Comprehensive test suite** with >90% code coverage
4. **Performance benchmarks** for parsing large programs
5. **Documentation** for AST structure and parser usage
6. **Integration tests** with lexer and future compiler phases

## Estimated Timeline

- **Phase 1**: 2-3 weeks (Expression parsing foundation)
- **Phase 2**: 2-3 weeks (Statement parsing completion)
- **Phase 3**: 3-4 weeks (Type system integration)
- **Phase 4**: 4-5 weeks (Advanced language features)
- **Phase 5**: 3-4 weeks (Semantic analysis integration)

**Total Estimated Time**: 14-19 weeks for complete AST implementation

## Next Immediate Steps

1. **Start with Phase 1.2**: Implement function call expression parsing
2. **Add operator precedence**: Implement Pratt parser for proper precedence
3. **Enhance expression tests**: Add comprehensive expression parsing tests
4. **Begin statement parsing**: Start with variable declarations (`let`/`var`)
5. **Plan type system**: Design type expression AST nodes

The FOL AST completion requires systematic implementation of these phases, with particular attention to maintaining the existing sophisticated streaming and namespace systems while building robust parsing capabilities.