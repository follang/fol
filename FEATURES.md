# FOL Programming Language - Current Features

## Overview

FOL is a sophisticated programming language implemented in Rust with a modular architecture. The project uses a Cargo workspace with multiple specialized crates for different compilation phases.

## Architecture

### Cargo Workspace Structure
```
fol/
├── fol-types/          # Core type definitions and error handling
├── fol-stream/         # Sophisticated file streaming with .mod handling
├── fol-lexer/          # Multi-stage lexical analysis
├── fol-parser/         # Abstract Syntax Tree parser
├── fol-diagnostics/    # Structured error reporting
└── src/                # Main CLI application
```

## Core Features

### 1. Sophisticated File Streaming System

**Location**: `fol-stream/`

#### Multi-Source Character Streaming
- Unified character stream across multiple source files
- Precise location tracking (file, row, column) for every character
- Seamless transitions between source files during compilation
- Support for both single file and folder compilation

#### Innovative .mod Directory Handling
**Key Innovation**: Directories ending in `.mod` are automatically **SKIPPED** during compilation.

```
project/
├── main.fol                 # ✓ Included in compilation
├── utils.fol                # ✓ Included in compilation
├── regular_dir/             # ✓ Processed recursively
│   └── helper.fol           # ✓ Included in compilation
├── main.mod/                # ⚡ SKIPPED during traversal
│   ├── main.fol             # ✗ Not included in main compilation
│   ├── binding.nim          # ✗ Mixed file type
│   └── ffi.go               # ✗ Mixed file type
└── network.mod/             # ⚡ SKIPPED during traversal
    ├── client.fol           # ✗ Module-specific code
    └── server.fol           # ✗ Module-specific code
```

**Benefits**:
- Enables modular organization with mixed file types
- Supports native language bindings in separate modules
- Allows module-specific tooling and configuration
- Clean separation between main compilation and module-specific code

#### Advanced Path Resolution
- Automatic canonical path resolution
- Support for both absolute and relative paths
- Robust error handling for missing/invalid paths
- Flexible file vs folder detection and handling

### 2. Comprehensive Namespace System

**Location**: `fol-stream/src/lib.rs:408-456`

#### Directory-Based Namespaces
- **Folders act as namespaces**: `one/two.fol` → functions callable as `one::two::func()`
- **Root namespace**: Uses package name from `Cargo.toml`
- **Hierarchical structure**: Subdirectories create nested namespaces

#### Package Detection
- Automatic package name detection from `Cargo.toml`
- Walks up directory tree to find project root
- Fallback to directory name if no `Cargo.toml` found
- Support for explicit package name override

#### Namespace Examples
```
test_old/main/
├── main.fol                 → namespace: fol
├── var/
│   ├── let.fol             → namespace: fol::var  
│   └── var.fol             → namespace: fol::var
├── single/
│   ├── input1.fol          → namespace: fol::single
│   └── subpak/
│       └── input1.fol      → namespace: fol::single::subpak
└── var2/
    └── var.fol             → namespace: fol::var2
```

#### Namespace Validation
- Filters invalid directory names (containing dots, special characters)
- Ensures namespace components are valid identifiers
- Maintains clean namespace hierarchy
- Excludes `.mod` directories from namespace paths

### 3. Multi-Stage Lexical Analysis

**Location**: `fol-lexer/`

#### Four-Stage Lexing Pipeline
1. **Stage 0**: Raw character stream to basic tokens
2. **Stage 1**: Token refinement and classification
3. **Stage 2**: Advanced token processing
4. **Stage 3**: Final token preparation for parser

#### Token Types Supported
- **Keywords**: `use`, `ali`, `imp`, `seg`, `lab`, `def`, `let`, `var`, `if`, `else`, `while`, `for`, `return`, `true`, `false`
- **Identifiers**: Variable and function names
- **Literals**: 
  - Numbers (integers, floats)
  - Strings (with escape sequences)
  - Booleans
- **Symbols**: `+`, `-`, `*`, `/`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`
- **Brackets**: `(`, `)`, `{`, `}`, `[`, `]`
- **Delimiters**: `,`, `;`, `:`

#### Advanced Features
- **Precise location tracking**: Every token maintains source file, row, column information
- **Comment handling**: Single-line (`//`) and multi-line (`/* */`) comments
- **Bracket matching**: Automatic bracket pairing and validation
- **Unicode support**: Full Unicode character handling
- **Error recovery**: Robust error handling with detailed diagnostics

### 4. Abstract Syntax Tree Parser

**Location**: `fol-parser/`

#### Current Implementation Status
- **Declaration support**: Parser handles `use`, `var`, `fun`, and `pro` declarations.
- **Expression support**: Precedence-aware parsing for arithmetic, comparison, logical operators, calls, method calls, and assignments.
- **Receiver methods**: Parser accepts receiver-form routine declarations (`fun (Type)name(...)` / `pro (Type)name(...)`) and tracks receiver-qualified return signatures.
- **Receiver type validation**: Builtin/special language receiver types are rejected; receiver declarations require user-defined named types.
- **Control flow support**: `return`, `break`, `yeild`, `when`, `if`/`else if` lowering, and `loop` forms are parsed into AST nodes.
- **Unary operators**: `-`, `+`, `not`, `&` (ref), and `*` (deref) are parsed with recursive prefix chaining support.
- **Error locations**: Parser errors carry file/line/column/length metadata into diagnostics.
- **Unary diagnostics**: Missing unary operands produce explicit diagnostics (including delimiter and EOF cases).
- **Literal helper**: `parse_literal` supports integers, strings, and identifiers for baseline parser tests.
- **Custom-error report checks**: Parser validates `report` arity/type compatibility against declared routine error type, including identifier/literal/expression/routine-call/method-call paths with explicit unknown routine/method diagnostics.
- **Forward declaration support in report checks**: Report expression typing resolves routine/method return signatures even when callee declarations appear later in the same file.
- **Signature conflict diagnostics**: Parser reports conflicting duplicate routine/method return signatures for the same callable key (`fun`/`pro`, plain and receiver-qualified), while allowing duplicates when return type is consistent.

#### In Progress
- Remaining declaration work (`typ`, `ali`, and advanced declaration options)
- Broader recovery strategy for malformed declaration and block boundaries
- Continued parser normalization/refactoring as feature coverage grows

### 5. Structured Diagnostic System

**Location**: `fol-diagnostics/`

#### Dual Output Formats
- **Human-readable**: Colored, formatted error messages for developers
- **JSON structured**: Machine-readable format for tools and IDEs

#### CLI Integration
```bash
# Human-readable diagnostics (default)
fol project/

# JSON structured output for tools
fol project/ --json
```

#### Diagnostic Features
- **Error categorization**: Syntax, semantic, type errors
- **Precise location reporting**: File, line, column information
- **Severity levels**: Error, warning, info, hint
- **Contextual information**: Code snippets and suggestions
- **Batch reporting**: Multiple diagnostics in single output

### 6. Comprehensive Type System

**Location**: `fol-types/`

#### Core Types
- **Basic types**: Integer, Float, String, Boolean
- **Collection types**: List, Vector, Array
- **Advanced types**: Function types, Generic types
- **Error types**: Structured error hierarchy

#### Error Hierarchy
```rust
pub trait Glitch: std::error::Error + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn Glitch>;
    fn as_any(&self) -> &dyn std::any::Any;
}
```

#### Type Categories
- **`Flaw`**: Syntax and parsing errors
- **`Typo`**: Type system errors
- **`Slip`**: Runtime and logic errors

### 7. Advanced CLI Interface

**Location**: `src/main.rs`

#### Command-Line Features
- **File compilation**: `fol main.fol`
- **Folder compilation**: `fol project/` (with sophisticated .mod handling)
- **JSON output**: `fol project/ --json`
- **Flexible input**: Automatic file vs folder detection

#### User Experience
- **Clear success messages**: "✓ Compilation successful!"
- **Detailed error reporting**: Precise location and context
- **Progress indication**: Compilation status and file processing
- **Colored output**: Enhanced readability with syntax highlighting

## Testing Infrastructure

### Comprehensive Test Suite
- **Stream tests**: 15+ tests covering file streaming, location tracking, Unicode handling
- **Lexer tests**: 12+ tests for token generation, error handling, performance
- **Parser tests**: 4+ tests for AST generation and validation
- **Namespace tests**: 8+ tests for namespace functionality
- **Integration tests**: 3+ end-to-end pipeline tests
- **Performance tests**: Large file handling and optimization

### Test Organization
```
test/
├── stream/                 # File streaming tests
├── lexer/                  # Lexical analysis tests
├── parser/                 # AST parsing tests
└── run_tests.rs           # Integration test runner
```

### Test Coverage
- **Unit tests**: Individual component testing
- **Integration tests**: Cross-component pipeline testing
- **Performance tests**: Large file and optimization testing
- **Error handling tests**: Robust error scenario coverage

## Performance Characteristics

### Optimizations
- **Lazy evaluation**: Efficient character streaming
- **Memory management**: Optimized token and AST node allocation
- **Concurrent processing**: Multi-threaded compilation pipeline
- **Caching**: Smart caching of parsed results

### Benchmarks
- **Large file handling**: Successfully processes 1000+ line files
- **Multi-source streaming**: Efficient handling of multiple source files
- **Memory usage**: Optimized memory footprint for large projects
- **Compilation speed**: Fast compilation times for typical projects

## Language Features

### Syntax Highlights
- **Clean syntax**: Readable and expressive language design
- **Type safety**: Strong static typing with inference
- **Memory safety**: Rust-based implementation ensures memory safety
- **Error handling**: Comprehensive error handling and recovery
- **Modularity**: Sophisticated module system with namespace support

### Code Examples
```fol
// Function definition
def fibonacci(n: Int) -> Int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Variable declarations
let x = 42;
var y = "hello world";

// Namespace usage
use math::vector::Vector3;
def main() {
    let v = Vector3::new(1.0, 2.0, 3.0);
}
```

## Current Status

### Implemented ✅
- Complete file streaming with sophisticated .mod handling
- Full namespace system with directory-based namespaces
- Multi-stage lexical analysis with comprehensive token support
- Basic AST parser scaffold with location-aware parser errors
- Structured diagnostic system with dual output formats
- Comprehensive type system with error hierarchy
- Advanced CLI with flexible input handling
- Extensive test coverage with multiple test categories

### Stable Features ✅
- Cargo workspace architecture
- Multi-source file compilation
- Location tracking throughout pipeline
- Error handling and recovery
- Unicode and international character support
- Performance optimizations for large projects

The FOL programming language has a solid streaming/lexing foundation and is actively transitioning the parser from scaffold mode to full AST construction.
