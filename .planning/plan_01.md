# FOL Language Implementation Plan

## Current State Analysis

### Repository Structure
```
/doc/code/fol/
├── src/
│   ├── ast/           # AST implementation
│   ├── syntax/        # Tokenizer and parser
│   ├── types/         # Type system
│   └── ...
├── book/              # Language specification
├── test/              # Test files
└── AST_IMPLEMENTATION.md  # AST design documentation
```

### Language Specification Overview

#### 1. General Structure
The FOL language follows this pattern:
```
declaration[options] name: type[options] = { implementation; }
```

#### 2. Top-Level Declarations
- `use` - Import statements
- `def` - Definitions/macros
- `seg` - Segments
- `var` - Variables
- `con` - Constants  
- `fun` - Functions
- `pro` - Procedures
- `typ` - Type definitions
- `ali` - Aliases
- `imp` - Implementations
- `lab` - Labels

#### 3. Type System

##### Ordinal Types
- `int[options]` - Integers (u8, u16, u32, u64, u128, 8, 16, 32, 64, 128, arch, uarch)
- `flt[options]` - Floating point (32, 64, arch)
- `chr[options]` - Characters (utf8, utf16, utf32)
- `bol` - Boolean

##### Container Types
- `arr[type,size]` - Static arrays
- `vec[type]` - Dynamic arrays (vectors)
- `seq[type]` - Sequences (linked lists)
- `mat[sizes...]` - Matrices (SIMD)
- `set[types...]` - Sets/tuples
- `map[key,value]` - Maps/dictionaries
- `axi[types...]` - Axioms (logic programming)

##### Complex Types
- `rec` - Records/structs
- `ent` - Entities
- `rut` - Routines (function types)
- `box` - Boxed types

##### Special Types
- `opt[type]` - Optional types
- `mul[types...]` - Multiple types (union)
- `any` - Any type
- `ptr[type]` - Pointers
- `err[type]` - Error types
- `non` - Never type

#### 4. Standards (Protocols/Blueprints)
- Protocol definitions for interfaces
- Blueprint definitions for implementations

#### 5. Generics
- Generic type parameters with constraints
- Associated types

#### 6. Memory Model
- Ownership system
- Borrowing (`&` and `&mut`)
- Move semantics

#### 7. Control Flow
- `when` - Pattern matching
- `loop` - Loops
- `if` - Conditionals
- `for`/`each` - Iteration
- `break`/`return`/`yield` - Flow control

#### 8. Sugar Features
- Pipes (`|>`)
- Silents (implicit universally quantified variables)
- Method chaining

## Current Implementation Status

### ✅ Implemented

1. **Token System** (`src/syntax/token/`)
   - Basic token types (keywords, symbols, literals)
   - Built-in keywords enumeration
   - Location tracking

2. **Basic Parser** (`src/syntax/parse/`)
   - Function parsing (limited to integer bodies)
   - Basic top-level declaration recognition
   - Whitespace and comment handling

3. **AST Structure** (`src/ast/`)
   - Comprehensive AstNode enum
   - FolType system definition
   - Declaration options structure

4. **Type System Foundation** (`src/types/`)
   - Error handling
   - Basic type definitions

### ❌ Not Implemented

1. **Parser Components**
   - [ ] Type declaration parsing (`typ name: rec/ent = { ... }`)
   - [ ] Variable/constant declaration parsing
   - [ ] Procedure parsing
   - [ ] Use declaration body parsing
   - [ ] Alias declaration parsing
   - [ ] Implementation declaration parsing
   - [ ] Segment declaration parsing
   - [ ] Definition/macro parsing
   - [ ] Label parsing

2. **Type Parsing**
   - [ ] Container type parsing (arr, vec, seq, mat, set, map, axi)
   - [ ] Complex type parsing (rec, ent, rut, box)
   - [ ] Special type parsing (opt, mul, any, ptr, err, non)
   - [ ] Generic type parameters
   - [ ] Type constraints

3. **Expression Parsing**
   - [ ] Binary operators
   - [ ] Unary operators
   - [ ] Function calls
   - [ ] Method calls
   - [ ] Field access
   - [ ] Array/container indexing
   - [ ] Literals (beyond integers)

4. **Statement Parsing**
   - [ ] Assignments
   - [ ] Control flow (when, loop, if)
   - [ ] Pattern matching
   - [ ] Return/break/yield statements

5. **Advanced Features**
   - [ ] Generics parsing
   - [ ] Standards (protocols/blueprints)
   - [ ] Memory model annotations
   - [ ] Sugar features (pipes, silents)

## Implementation Plan

### Phase 1: Complete Top-Level Declaration Parsing
**Goal**: Parse all top-level declarations according to FOL spec

1. **Implement Type Declaration Parser** (typ)
   - Parse `typ name: rec = { fields }`
   - Parse `typ name: ent = { fields }`
   - Support generics and options

2. **Implement Variable/Constant Parsers**
   - Parse `var[options] name: type = value`
   - Parse `con[options] name: type = value`
   - Handle all type specifications

3. **Implement Procedure Parser**
   - Parse `pro[options] name(params): type = { body }`
   - Similar to function but different semantics

4. **Complete Use Declaration Parser**
   - Parse use declaration bodies properly
   - Handle aliasing and selective imports

5. **Implement Other Declarations**
   - Alias declarations (ali)
   - Implementation declarations (imp)
   - Segment declarations (seg)
   - Definition/macro declarations (def)
   - Label declarations (lab)

### Phase 2: Type System Parsing
**Goal**: Parse all FOL types correctly

1. **Ordinal Types**
   - Integer options parsing
   - Float options parsing
   - Character encoding options

2. **Container Types**
   - Array size specifications
   - Generic type parameters
   - Nested containers

3. **Complex Types**
   - Record field parsing
   - Entity member parsing
   - Routine type signatures

4. **Special Types**
   - Optional type wrapping
   - Multiple type unions
   - Pointer specifications

### Phase 3: Expression and Statement Parsing
**Goal**: Parse function/procedure bodies

1. **Expression Parser**
   - Operator precedence
   - Function/method calls
   - Field access and indexing

2. **Statement Parser**
   - Assignment statements
   - Control flow structures
   - Pattern matching

3. **Block and Scope Parsing**
   - Nested blocks
   - Variable scoping
   - Return values

### Phase 4: Advanced Features
**Goal**: Support full FOL specification

1. **Generics System**
   - Type parameters
   - Constraints
   - Associated types

2. **Standards System**
   - Protocol definitions
   - Blueprint implementations

3. **Memory Model**
   - Ownership annotations
   - Borrowing syntax
   - Lifetime parameters

4. **Sugar Features**
   - Pipe operators
   - Silent variables
   - Method chaining

## Testing Strategy

1. **Unit Tests**: Test each parser component individually
2. **Integration Tests**: Test complete FOL programs
3. **Error Cases**: Test parser error handling
4. **Performance Tests**: Ensure parser performance on large files

## Next Steps

1. Start with Phase 1 - implement type declaration parser
2. Create test cases for each declaration type
3. Incrementally add features while maintaining working parser
4. Document parser implementation as we go

## File Modifications Needed

1. `/src/syntax/parse/stat/assign/typ/mod.rs` - Enhance type parsing
2. `/src/syntax/parse/stat/assign/var/mod.rs` - Create variable parser
3. `/src/syntax/parse/stat/assign/con/mod.rs` - Create constant parser
4. `/src/syntax/parse/stat/assign/pro/mod.rs` - Create procedure parser
5. `/src/syntax/parse/stat/assign/use/mod.rs` - Enhance use parser
6. `/src/syntax/parse/stat/assign/ali/mod.rs` - Create alias parser
7. `/src/syntax/parse/stat/assign/imp/mod.rs` - Create implementation parser
8. `/src/syntax/parse/stat/assign/seg/mod.rs` - Create segment parser
9. `/src/syntax/parse/stat/assign/def/mod.rs` - Create definition parser
10. `/src/syntax/parse/stat/assign/lab/mod.rs` - Create label parser

## Current Parser Issues

1. Parser only handles functions with integer bodies
2. No support for complex types
3. No expression parsing beyond literals
4. No statement parsing beyond basic blocks
5. Limited error recovery

## Success Criteria

- [ ] Can parse `/test/main/main.fol` completely
- [ ] Generates correct AST for all constructs
- [ ] Provides helpful error messages
- [ ] Handles malformed input gracefully
- [ ] Performance is acceptable for large files