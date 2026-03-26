# Architecture

This document describes how the FOL compiler is organized, how crates
depend on each other, and how data flows from source to binary.

## Workspace layout

```

## Build Surface Layers

The public `build.fol` surface is intentionally layered.

Current public layering:

- `.build()`
  ambient package build context
- `build.meta({...})`
  package identity and package metadata
- `build.add_dep({...})`
  direct dependency declarations
- `build.graph()`
  artifact, step, option, and generated-file graph mutation

This split is intentional:

- package metadata is not graph mutation
- direct dependency declarations are not artifact declarations
- graph construction should not become a catch-all stringly package API

Near-term build-system expansion should stay inside this layering rather than
replace it:

- dependency handles returned from `build.add_dep({...})`
- unified output handles for generated and copied files
- explicit dependency build-argument forwarding
- cleaner install-prefix/output-root behavior

The design constraint is:

- richer build values on top of the current layers
- no return to YAML manifests
- no reintroduction of public `Graph`/`Build` type names
lang/
  compiler/       front-end: source --> typed IR
    fol-types         shared type definitions and traits
    fol-stream        file/directory to character-stream conversion
    fol-lexer         4-stage tokenization pipeline
    fol-parser        AST-only parser (no semantic analysis)
    fol-diagnostics   error formatting and diagnostic output
    fol-intrinsics    compiler-owned builtin operation registry
    fol-package       package identity, metadata, git, lockfiles
    fol-resolver      whole-program name resolution and scope graph
    fol-typecheck     type checking and inference for V1 subset
    fol-lower         typed AST to backend-oriented lowered IR

  execution/      back-end: IR --> binary + runtime
    fol-build         build graph IR, build.fol API, executor
    fol-runtime       runtime support library linked into output binaries
    fol-backend       Rust code generation and direct rustc invocation

  tooling/        user-facing tools
    fol-frontend      CLI entry point, workspace orchestration
    fol-editor        LSP server and tree-sitter integration
```

## Dependency graph

### Layered view

Every crate only depends on crates in the same layer or a layer above it.

```
LAYER 0 — foundations (no workspace dependencies)

  ┌───────────┐  ┌────────────┐  ┌─────────────────┐  ┌────────────────┐  ┌─────────────┐
  │ fol-types │  │ fol-stream │  │ fol-diagnostics │  │ fol-intrinsics │  │ fol-runtime │
  │           │  │            │  │                 │  │                │  │             │
  │ shared    │  │ files to   │  │ error display   │  │ builtin ops    │  │ containers, │
  │ traits,   │  │ character  │  │ and diagnostic  │  │ registry       │  │ strings,    │
  │ core defs │  │ streams    │  │ formatting      │  │ (.eq, .len,    │  │ ABI types   │
  │           │  │            │  │                 │  │  .echo, etc)   │  │ for output  │
  └───────────┘  └────────────┘  └─────────────────┘  └────────────────┘  │ binaries    │
       │              │                  │                    │           └─────────────┘
       │              │                  │                    │                   │
-------│--------------│------------------│--------------------│-------------------------
       │              │                  │                    │                   │
       ▼              ▼                  │                    │                   │
LAYER 1 — tokenization                   │                    │                   │
                                         │                    │                   │
  ┌──────────────────────────────────┐   │                    │                   │
  │           fol-lexer              │   │                    │                   │
  │                                  │   │                    │                   │
  │  characters --> tokens           │   │                    │                   │
  │  4-stage: stream -> chars ->     │   │                    │                   │
  │           tokens -> elements     │   │                    │                   │
  │                                  │   │                    │                   │
  │  deps: types, stream             │   │                    │                   │
  └──────────────────────────────────┘   │                    │                   │
       │                                 │                    │                   │
-------│---------------------------------│--------------------│-------------------│-----
       │                                 │                    │                   │
       ▼                                 ▼                    │                   │
LAYER 2 — syntax                                              │                   │
                                                              │                   │
  ┌─────────────────────────────────────────────────────┐     │                   │
  │                    fol-parser                       │     │                   │
  │                                                     │     │                   │
  │  tokens --> untyped AST                             │     │                   │
  │  declarations, expressions, types, control flow     │     │                   │
  │  no semantic analysis during parsing                │     │                   │
  │                                                     │     │                   │
  │  deps: types, stream, lexer, diagnostics            │     │                   │
  └─────────────────────────────────────────────────────┘     │                   │
       │                                                      │                   │
       ├──────────────────────────────────┐                   │                   │
       │                                  │                   │                   │
-------│----------------------------------│-------------------│-------------------│-----
       │                                  │                   │                   │
       ▼                                  ▼                   │                   │
LAYER 3 — build system + packages                             │                   │
                                                              │                   │
  ┌────────────────────────────┐ ┌───────────────────────────┐│                   │
  │        fol-build           │ │       fol-package         ││                   │
  │                            │ │                           ││                   │
  │  owns all build logic:     │ │  re-exports fol-build     ││                   │
  │   - build graph IR         │ │  adds package concerns:   ││                   │
  │   - build.fol API surface  │ │   - PackageIdentity       ││                   │
  │   - build.fol executor     │ │   - build.fol metadata    ││                   │
  │   - artifact definitions   │ │   - git fetch/clone       ││                   │
  │   - step ordering          │ │   - lockfile handling     ││                   │
  │   - option resolution      │ │   - build entry validation││                   │
  │   - dependency surfaces    │ │   - session/root discovery││                   │
  │   - native artifact decls  │ │                           ││                   │
  │   - codegen definitions    │ │  deps: build, diagnostics,││                   │
  │   - capability enforcement │ │        lexer, parser,     ││                   │
  │                            │ │        stream             ││                   │
  │  deps: diagnostics, lexer, │ └───────────────────────────┘│                   │
  │        parser, stream      │       ▲                      │                   │
  └────────────────────────────┘       │                      │                   │
       ▲                               │                      │                   │
       └───────────────────────────────┘                      │                   │
                                                              │                   │
--------------------------------------------------------------│-------------------│-----
       │                                                      │                   │
       ▼                                                      │                   │
LAYER 4 — name resolution                                     │                   │
                                                              │                   │
  ┌───────────────────────────────────────────────────────┐   │                   │
  │                    fol-resolver                       │   │                   │
  │                                                       │   │                   │
  │  AST + packages --> scoped name graph                 │   │                   │
  │  resolves imports across package boundaries           │   │                   │
  │  builds scope graph with visibility rules             │   │                   │
  │  uses build graph to understand declared modules/deps │   │                   │
  │                                                       │   │                   │
  │  deps: types, build, package, parser, diagnostics,    │   │                   │
  │        lexer, stream                                  │   │                   │
  └───────────────────────────────────────────────────────┘   │                   │
       │                                                      │                   │
-------│------------------------------------------------------│-------------------│-----
       │                                                      │                   │
       ▼                                                      ▼                   │
LAYER 5 — type checking                                                           │
                                                                                  │
  ┌──────────────────────────────────────────────────────┐                        │
  │                   fol-typecheck                      │                        │
  │                                                      │                        │
  │  resolved names --> typed workspace                  │                        │
  │  type inference and checking for V1 subset           │                        │
  │  scalars, records, entries, containers, routines     │                        │
  │  error types, shell types, conversions               │                        │
  │                                                      │                        │
  │  deps: diagnostics, intrinsics, parser, resolver     │                        │
  └──────────────────────────────────────────────────────┘                        │
       │                                                                          │
-------│--------------------------------------------------------------------------│-----
       │                                                                          │
       ▼                                                                          │
LAYER 6 — lowering                                                                │
                                                                                  │
  ┌──────────────────────────────────────────────────────────┐                    │
  │                     fol-lower                            │                    │
  │                                                          │                    │
  │  typed workspace --> lowered IR                          │                    │
  │  produces: LoweredWorkspace                              │                    │
  │    - LoweredPackage    (package grouping)                │                    │
  │    - LoweredRoutine    (functions/procedures with CFG)   │                    │
  │    - LoweredBlock      (basic blocks with instructions)  │                    │
  │    - LoweredInstr      (individual operations)           │                    │
  │    - LoweredTerminator (jump, branch, return)            │                    │
  │    - LoweredTypeDecl   (type definitions)                │                    │
  │    - LoweredSourceMap  (source location tracking)        │                    │
  │                                                          │                    │
  │  deps: diagnostics, intrinsics, parser, resolver,        │                    │
  │        typecheck                                         │                    │
  └──────────────────────────────────────────────────────────┘                    │
       │                                                                          │
-------│--------------------------------------------------------------------------│-----
       │                                                                          │
       ▼                                                                          ▼
LAYER 7 — code generation

  ┌──────────────────────────────────────────────────────────────────────────────────┐
  │                              fol-backend                                         │
  │                                                                                  │
  │  lowered IR + build graph --> Rust source --> rustc --> binary                   │
  │                                                                                  │
  │  1. receives lowered IR from fol-lower                                           │
  │  2. receives build graph from fol-build (via fol-frontend)                       │
  │  3. emits a temporary Rust crate from the lowered IR                             │
  │  4. compiles fol-runtime into .rlib with direct rustc                            │
  │  5. compiles the generated crate with direct rustc                               │
  │  6. copies binary to output directory                                            │
  │                                                                                  │
  │  includes: name mangling, layout planning, type rendering,                       │
  │            terminator rendering, control flow generation                         │
  │                                                                                  │
  │  deps: intrinsics, lower, parser, resolver, runtime                              │
  └──────────────────────────────────────────────────────────────────────────────────┘
       │
       ▼
  binary artifact (.fol/build/<profile>/bin/<target>/<name>)
```

### Tooling layer

The tooling crates sit beside the pipeline and reach into multiple layers.

```
  ┌──────────────────────────────────────────────────────────────────┐
  │                        fol-frontend                              │
  │                                                                  │
  │  the fol CLI binary                                              │
  │  orchestrates the full pipeline: parse -> resolve -> check ->    │
  │  lower -> backend                                                │
  │  also hosts: workspace discovery, build evaluation,              │
  │              command dispatch (build, run, test, check, edit)    │
  │                                                                  │
  │  deps: package, stream, lexer, parser, resolver, typecheck,      │
  │        lower, backend, editor, diagnostics                       │
  └──────────────────────────────────────────────────────────────────┘
       │
       │ embeds
       ▼
  ┌──────────────────────────────────────────────────────────────────┐
  │                         fol-editor                               │
  │                                                                  │
  │  LSP server: diagnostics, hover, go-to-definition, symbols       │
  │  tree-sitter syntax layer for editor highlighting                │
  │  runs the compiler pipeline up to typecheck for diagnostics      │
  │                                                                  │
  │  deps: diagnostics, intrinsics, lexer, package, parser,          │
  │        resolver, stream, typecheck, types                        │
  └──────────────────────────────────────────────────────────────────┘
```

## Data flow

How a FOL source file becomes a binary:

```
  *.fol source files                         build.fol
       │                                        │
       ▼                                        ▼
  ┌──────────┐                              ┌───────────┐
  │fol-stream│  read files into             │fol-package│  parse package
  │          │  character streams           │           │  metadata,
  └────┬─────┘                              │           │  dependencies,
       │                                    │           │  and identity
       ▼                                    └─────┬─────┘
  ┌──────────┐                                    │
  │fol-lexer │  chars --> tokens                  │
  │          │  (4-stage pipeline)                │
  └────┬─────┘                                    │
       │                                          │
       ▼                                          │
  ┌──────────┐                                    │
  │fol-parser│  tokens --> AST                    │
  │          │                                    │
  └────┬─────┘                                    │
       │                                          │
       ├── ordinary source ─┐                     │
       │                    │                     │
       └─── build.fol ─┐    │                     │
                       │    │                     │
                       ▼    ▼                     │
                    ┌──────────┐                  │
                    │fol-build │                  │
                    │          │ evaluate         │
                    │          │ build.fol into   │
                    │          │ metadata, deps,  │
                    │          │ and build graph: │
                    │          │  - artifacts     │
                    │          │  - steps         │
                    │          │  - options       │
                    │          │  - dependencies  │
                    │          │  - native decls  │
                    │          │  - codegen defs  │
                    └────┬─────┘                  │
                         │                        │
                         ▼                        ▼
                    ┌───────────────────────────────────┐
                    │         fol-resolver              │
                    │                                   │
                    │  AST + packages + build graph     │
                    │  --> resolved name graph          │
                    │                                   │
                    │  imports, scopes, visibility,     │
                    │  cross-package references         │
                    └──────────────┬────────────────────┘
                                   │
                                   ▼
                    ┌───────────────────────────────────┐
                    │         fol-typecheck             │
                    │                                   │
                    │  resolved workspace               │
                    │  --> typed workspace              │
                    │                                   │
                    │  inference, checking, coercions,  │
                    │  error type validation            │
                    └──────────────┬────────────────────┘
                                   │
                                   ▼
                    ┌───────────────────────────────────┐
                    │          fol-lower                │
                    │                                   │
                    │  typed workspace                  │
                    │  --> LoweredWorkspace             │
                    │                                   │
                    │  packages, routines, blocks,      │
                    │  instructions, terminators,       │
                    │  type declarations, source maps   │
                    └──────────────┬────────────────────┘
                                   │
                                   ▼
              ┌──────────────────────────────────────────────┐
              │              fol-backend                     │
              │                                              │
              │  lowered IR ──> emit Rust source             │
              │  build graph ──> select artifact + target    │
              │                                              │
              │  rustc fol-runtime --> .rlib                 │
              │  rustc generated crate + .rlib --> binary    │
              │                                              │
              │  (no Cargo — direct rustc invocation)        │
              └────────────────────┬─────────────────────────┘
                                   │
                                   ▼
                    ┌───────────────────────────────────┐
                    │       output binary               │
                    │                                   │
                    │  .fol/build/<profile>/bin/        │
                    │         <target>/<name>           │
                    │                                   │
                    │  links against fol-runtime .rlib  │
                    └───────────────────────────────────┘
```

## Key relationships

### fol-build vs fol-package

`fol-build` owns all build system logic: the build graph IR, the
`build.fol` API surface (`graph.add_exe()`, `graph.add_run()`, etc.),
the executor that interprets `build.fol`, artifact definitions, step
ordering, option resolution, dependency surfaces, native artifact
declarations, codegen definitions, and capability enforcement.

`fol-package` depends on `fol-build` and re-exports everything from it
through thin shim modules (each `build_*.rs` file is a single
`pub use fol_build::*` line). On top of that, `fol-package` adds its
own package-level concerns: `PackageIdentity`, `PackageMetadata`
(from `build.fol`), git fetching, lockfile handling, package session
and root discovery, and build entry validation.

Downstream crates import through `fol-package` as a single entry point.

### fol-resolver reaches into build and package

`fol-resolver` depends on both `fol-build` and `fol-package` because
name resolution must understand:

- package boundaries and source kinds (entry, local, std, installed)
- build-declared modules and their root paths
- inter-package dependency surfaces and export mounts
- how `use loc`, `use std`, and `use pkg` imports map to real packages

Without the build graph, the resolver cannot know which modules exist
or how packages expose their namespaces.

### fol-runtime is standalone

`fol-runtime` has zero workspace dependencies. It is a pure Rust
library that gets compiled separately by `fol-backend` using `rustc`
and linked into the output binary as an `.rlib`. It provides:

- container types (vec, seq, set, map)
- string support
- shell types (opt, err)
- error handling / recovery ABI (`FolRecover`)
- builtin operation implementations

Generated Rust code calls into `fol-runtime` types and functions.

### fol-backend drives rustc directly

The backend does not use Cargo for artifact builds. The build flow is:

1. emit a temporary Rust crate from lowered IR
2. compile `fol-runtime` into an `.rlib` with `rustc --crate-type=lib`
3. compile the generated entry crate with `rustc`, linking against the
   runtime `.rlib`
4. copy the resulting binary to the output directory

All `rustc` calls use the target triple from the build graph when
cross-compiling.

### fol-editor runs a partial pipeline

The LSP server in `fol-editor` runs the compiler pipeline up through
`fol-typecheck` to produce diagnostics, hover information, go-to-
definition, and document symbols. It does not invoke lowering or the
backend — only the front-end phases needed for editor feedback.

### fol-frontend orchestrates everything

`fol-frontend` is the only crate that depends on nearly every other
crate. It is the CLI binary (`fol`) and the orchestration point that
wires the full pipeline together based on the user's command
(`build`, `run`, `test`, `check`, `edit`).

## Version milestones

See `plan/VERSIONS.md` for the full rationale.

- **V1** — core language: scalars, records, entries, functions,
  procedures, control flow, basic containers, error handling, aliases.
  The compiler pipeline is complete end-to-end for this subset.

- **V2** — advanced semantics: generics, standards/protocols,
  blueprints, contracts, metaprogramming, advanced type inference.
  Requires conformance checking and constraint machinery.

- **V3** — systems interop: ownership, borrowing, pointers,
  concurrency (eventuals, coroutines), C ABI, native linking, foreign
  declarations. Requires runtime model decisions and linker integration.
