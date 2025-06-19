# FOL AST Implementation

This document explains the complete AST (Abstract Syntax Tree) implementation for the FOL programming language.

## Overview

The AST implementation provides a proper semantic representation of FOL programs, replacing the previous "pretty-printer" nodes with a structured tree that captures the language's semantics and enables type checking, code generation, and other compiler phases.

## Design Philosophy

The AST design follows these principles:

1. **Semantic Representation**: Nodes store semantic information rather than just display strings
2. **Type Safety**: Strong typing throughout the AST with comprehensive type system support
3. **FOL Language Compliance**: Direct mapping to FOL's unique language features
4. **Visitor Pattern**: Easy traversal for analysis and code generation
5. **Memory Efficiency**: Minimal memory overhead while maintaining rich semantic information

## Core Components

### 1. AstNode Enum

The `AstNode` enum is the central type representing all possible AST nodes:

```rust
pub enum AstNode {
    // Declarations
    VarDecl, FunDecl, ProDecl, TypeDecl, UseDecl, AliasDecl,
    
    // Expressions  
    BinaryOp, UnaryOp, FunctionCall, MethodCall, 
    IndexAccess, FieldAccess, Identifier, Literal,
    ContainerLiteral, Range,
    
    // Statements
    Assignment, When, Loop, Return, Break, Yield,
    
    // Compound
    Block, Program
}
```

### 2. FOL Type System (`FolType`)

The type system captures FOL's rich type features:

#### Ordinal Types
- `Int { size: Option<IntSize>, signed: bool }` - FOL integers (int[u32], int[64], etc.)
- `Float { size: Option<FloatSize> }` - FOL floats (flt[32], flt[64])
- `Char { encoding: CharEncoding }` - FOL characters (chr[utf8])
- `Bool` - FOL booleans (bol)

#### Container Types
- `Array { element_type, size }` - Fixed arrays (arr[int, 5])
- `Vector { element_type }` - Dynamic vectors (vec[str])
- `Sequence { element_type }` - Linked lists (seq[int])
- `Matrix { element_type, dimensions }` - SIMD matrices (mat[3,3])
- `Set { types }` - Heterogeneous tuples (set[str, int, bool])
- `Map { key_type, value_type }` - Hash maps (map[str, int])

#### Complex Types
- `Record { fields }` - Structs/records (typ user: rec)
- `Entry { variants }` - Enums/entries (typ color: ent)

#### Special Types
- `Optional { inner }` - Optional types (opt[int])
- `Multiple { types }` - Union types (mul[int, str])
- `Pointer { target }` - Pointer types (ptr[int])
- `Function { params, return_type }` - Function types

### 3. Declaration Options

Each declaration type has specific options that map to FOL's option system:

#### Variable Options (`VarOption`)
```rust
pub enum VarOption {
    Mutable,     // mut or ~
    Static,      // sta or !
    Reactive,    // rac or ?
    Export,      // exp or +
    Hidden,      // hid or -
    New,         // allocate on heap
    Borrowing,   // bor - borrowing
}
```

#### Function Options (`FunOption`)
```rust
pub enum FunOption {
    Export,      // exp or +
    Mutable,     // mut
    Iterator,    // itr
}
```

### 4. Control Flow

#### When Statements
FOL's `when` statement supports multiple matching patterns:

```rust
pub enum WhenCase {
    Case { condition, body },    // case(condition) {}
    Is { value, body },         // is(value) {}
    In { range, body },         // in(range) {}
    Has { member, body },       // has(member) {}
    Of { type_match, body },    // of(type) {}
    On { channel, body },       // on(channel) {}
}
```

#### Loop Statements
```rust
pub enum LoopCondition {
    Condition(AstNode),                                    // loop(condition)
    Iteration { var, iterable, condition },               // loop(var in iterable)
}
```

## Key Features

### 1. Type Inference Support

The AST provides type inference capabilities:

```rust
impl AstNode {
    pub fn get_type(&self) -> Option<FolType> {
        match self {
            AstNode::Literal(Literal::Integer(_)) => Some(FolType::Int { size: None, signed: true }),
            AstNode::BinaryOp { op: BinaryOperator::Add, left, right } => {
                // Type promotion logic for arithmetic
                left.get_type().or_else(|| right.get_type())
            },
            // ... more cases
        }
    }
}
```

### 2. Tree Traversal

Easy navigation through the AST:

```rust
impl AstNode {
    pub fn children(&self) -> Vec<&AstNode> {
        // Returns all child nodes for tree traversal
    }
}
```

### 3. Visitor Pattern

The visitor pattern enables easy AST analysis:

```rust
pub trait AstVisitor {
    fn visit(&mut self, node: &AstNode);
    fn visit_var_decl(&mut self, options: &[VarOption], name: &str, type_hint: &Option<FolType>, value: &Option<Box<AstNode>>);
    fn visit_fun_decl(&mut self, options: &[FunOption], name: &str, params: &[Parameter], return_type: &Option<FolType>, body: &[AstNode]);
    // ... more visit methods
}
```

## Parser Implementation

The `AstParser` builds proper AST nodes with:

### 1. Operator Precedence

Proper operator precedence parsing:
- Logical OR (`or`)
- Logical AND (`and`) 
- Equality (`==`, `!=`)
- Comparison (`>`, `<`, `>=`, `<=`)
- Addition/Subtraction (`+`, `-`)
- Multiplication/Division (`*`, `/`, `%`)
- Unary operators (`-`, `not`)

### 2. Expression Parsing

Comprehensive expression support:
- Literals (integers, floats, strings, booleans)
- Identifiers
- Function calls
- Array/field access
- Binary and unary operations
- Parenthesized expressions
- Container literals

### 3. Declaration Parsing

Full support for FOL declarations:
- `var[options] name: type = value`
- `fun[options] name(params): return_type = { body }`
- `pro[options] name(params): return_type = { body }`
- `typ[options] name: definition`
- `use[options] name: type = { path }`
- `ali name: target_type`

## Usage Examples

### Parsing a Variable Declaration
```fol
var[mut, exp] counter: int[32] = 0
```

Creates AST:
```rust
AstNode::VarDecl {
    options: vec![VarOption::Mutable, VarOption::Export],
    name: "counter".to_string(),
    type_hint: Some(FolType::Int { size: Some(IntSize::I32), signed: true }),
    value: Some(Box::new(AstNode::Literal(Literal::Integer(0)))),
}
```

### Parsing a Function Declaration
```fol
fun[exp] add(a, b: int): int = {
    return (a + b)
}
```

Creates AST:
```rust
AstNode::FunDecl {
    options: vec![FunOption::Export],
    generics: vec![],
    name: "add".to_string(),
    params: vec![
        Parameter { name: "a".to_string(), param_type: FolType::Int { size: None, signed: true }},
        Parameter { name: "b".to_string(), param_type: FolType::Int { size: None, signed: true }},
    ],
    return_type: Some(FolType::Int { size: None, signed: true }),
    body: vec![
        AstNode::Return {
            value: Some(Box::new(AstNode::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(AstNode::Identifier { name: "a".to_string() }),
                right: Box::new(AstNode::Identifier { name: "b".to_string() }),
            }))
        }
    ],
}
```

## Benefits Over Previous Implementation

### 1. Semantic Richness
- **Before**: Nodes stored display strings (`string: String`)  
- **After**: Nodes store semantic information (types, options, structure)

### 2. Type Information
- **Before**: No type information available
- **After**: Full type system integration with inference support

### 3. Analysis Capability
- **Before**: Only pretty-printing possible
- **After**: Type checking, optimization, code generation enabled

### 4. FOL Language Support
- **Before**: Generic AST structure
- **After**: Direct mapping to FOL's unique features (when statements, borrowing, etc.)

### 5. Error Recovery
- **Before**: Basic error handling
- **After**: Structured error recovery with precise error locations

## Integration with Compiler Pipeline

The AST integrates seamlessly with other compiler phases:

1. **Lexer** → **Parser** → **AST** ✅ (Implemented)
2. **AST** → **Semantic Analysis** (Next phase)
3. **AST** → **Type Checker** (Next phase)
4. **AST** → **Code Generator** (Future phase)

## Testing

The AST can be tested with the existing FOL test files:

```rust
// Example test
let mut parser = AstParser::new();
let ast = parser.parse(&mut tokens).unwrap();

match ast {
    AstNode::Program { declarations } => {
        // Verify declarations match expected structure
        assert_eq!(declarations.len(), expected_count);
    }
    _ => panic!("Expected program node"),
}
```

## Future Enhancements

1. **Symbol Table Integration**: Link identifiers to their declarations
2. **Scope Analysis**: Track variable scoping and lifetime
3. **Generic Instantiation**: Handle generic type instantiation
4. **Macro Expansion**: Support for FOL's macro system
5. **Optimization Passes**: AST-level optimizations

## Conclusion

This AST implementation provides a solid foundation for the FOL compiler, capturing the language's rich semantics while enabling sophisticated analysis and code generation. The design directly reflects FOL's unique features like borrowing, when statements, and the comprehensive type system, making it a true semantic representation rather than just a parse tree.
