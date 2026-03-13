# FOL Package Plan

Last rebuilt: 2026-03-13
Scope: `fol-package` plus the minimal resolver/CLI/doc refactors required to move package acquisition and build-definition handling out of `fol-resolver`

## 0. Purpose

- `fol-stream`, `fol-lexer`, `fol-parser`, and `fol-resolver` are already in good shape for the current front-end and name-resolution contract.
- The next structural problem is package management and package acquisition.
- Today, `fol-resolver` still owns too much package-specific work:
- parsing `package.yaml`
- parsing `build.fol`
- validating package roots
- loading package-store roots
- preloading transitive package dependencies
- That work belongs in a dedicated crate.
- This plan defines the new crate, its boundaries, its execution role, the migration out of `fol-resolver`, the test strategy, and the first forward-compatibility hooks for future C ABI work.

## 1. Main Decision

We add a new crate named `fol-package`.

It sits before `fol-resolver` in the compile pipeline, but it still depends on the existing front-end crates because it must parse `build.fol`.

Execution order:

`entry discovery -> fol-package -> fol-resolver -> later semantic phases`

Internal crate dependency direction:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver`

That means:

- `fol-package` may use the lexer/parser to understand package control files
- `fol-resolver` must stop owning package metadata/build parsing and package-store loading
- `fol-resolver` should consume prepared package data from `fol-package`

## 2. Target Package Contract

### 2.1 Import Kinds

- `loc`: local manifest-free directory import
- `std`: standard-library directory import rooted under an explicit std root
- `pkg`: external or cross-package import backed by package control files and package acquisition

### 2.2 Directory Model

- FOL still treats directories as package/namespace units.
- Files are connected source units inside one directory-backed package surface.
- `use` never imports a single file as an independent module concept.
- A direct `.fol` file target for `loc`, `std`, or `pkg` remains invalid.

### 2.3 Control Files

- `package.yaml`: metadata only
- `build.fol`: dependency and export definitions

Rules:

- `loc` requires neither `package.yaml` nor `build.fol`
- `std` requires neither `package.yaml` nor `build.fol` for the current stdlib phase
- `pkg` requires both `package.yaml` and `build.fol`
- stray `package.fol` files are ignored

### 2.4 Consumer vs Definition Syntax

- `use` is consumer-side syntax in ordinary source files
- `def` remains a general FOL declaration form in ordinary user code and in `build.fol`
- `build.fol` uses ordinary FOL syntax; it is not a separate mini-language
- package definitions should come from recognized declarations inside `build.fol`, not from overloading `use`
- ordinary source files must not become package-definition surfaces just because `def` exists there too

## 3. What `fol-package` Must Own

`fol-package` is responsible for:

- package metadata loading from `package.yaml`
- package build-definition loading from `build.fol`
- package root validation
- `loc`, `std`, and `pkg` directory/materialization logic
- package identity and package graph modeling
- package cache/store interaction
- future source acquisition for `pkg`
- future lockfile/store-key behavior
- exported-root selection from `build.fol`
- package-level cycle detection during loading/materialization

`fol-package` is not responsible for:

- name resolution inside ordinary source code
- scope construction for ordinary code
- symbol/reference graphs
- type checking
- ownership or borrow analysis
- backend lowering
- final linking

## 4. What Must Stay In `fol-resolver`

`fol-resolver` still owns:

- lowering `use` declarations into import records
- plain and qualified name resolution
- visibility enforcement such as `hid`
- package/namespace/file scope resolution rules
- imported-symbol exposure rules
- unresolved / duplicate / ambiguity diagnostics for ordinary code

The resolver should ask `fol-package` for prepared package roots. It should not:

- read `package.yaml`
- parse `build.fol`
- scan package-store directories directly
- own package acquisition caches

## 5. Pipeline Shape

There are really two related pipelines now.

### 5.1 Package-Control Pipeline

Used for `build.fol` and package acquisition:

`filesystem -> fol-stream -> fol-lexer -> fol-parser -> fol-package`

### 5.2 Ordinary Source Pipeline

Used for user code:

`filesystem -> fol-stream -> fol-lexer -> fol-parser -> fol-package provider -> fol-resolver`

Important clarification:

- `fol-package` is before `fol-resolver` in execution
- `fol-package` is not before the parser in dependency order, because it reuses the parser to read `build.fol`

## 6. Current State To Migrate Out Of Resolver

The following resolver-owned areas should move into `fol-package`:

- [fol-resolver/src/manifest.rs](/home/bresilla/data/code/bresilla/fol/fol-resolver/src/manifest.rs)
- [fol-resolver/src/build_definition.rs](/home/bresilla/data/code/bresilla/fol/fol-resolver/src/build_definition.rs)
- package root loading and caching in [fol-resolver/src/session.rs](/home/bresilla/data/code/bresilla/fol/fol-resolver/src/session.rs)
- package-store-root and std-root loading policy currently wired through resolver config

The migration target is:

- `fol-package` loads and prepares packages
- `fol-resolver` resolves already-prepared packages

## 7. Proposed Crate Shape

Workspace addition:

- `fol-package`

Expected crate surface:

- `fol-package/src/lib.rs`
- `fol-package/src/errors.rs`
- `fol-package/src/config.rs`
- `fol-package/src/identity.rs`
- `fol-package/src/metadata.rs`
- `fol-package/src/build.rs`
- `fol-package/src/model.rs`
- `fol-package/src/session.rs`
- `fol-package/src/store.rs`
- `fol-package/src/fetch.rs`
- `fol-package/src/providers.rs`

Some files can collapse or move, but these responsibilities must exist.

## 8. Core Data Model

### 8.1 PackageConfig

Likely shape:

- `std_root: Option<String>`
- `package_store_root: Option<String>`
- `package_cache_root: Option<String>`
- future fetch/install flags as needed

This should replace resolver-owned package-loading config.

### 8.2 PackageSourceKind

Required families:

- `Entry`
- `Local`
- `Standard`
- `Package`

This should move out of `fol-resolver`.

### 8.3 PackageIdentity

Each prepared package needs:

- `source_kind`
- `canonical_root`
- `display_name`
- optional future source locator / resolved revision

### 8.4 PackageMetadata

Parsed only from `package.yaml`.

Initial fields:

- `name`
- `version`
- optional `kind`
- optional `description`
- optional `license`

This stays intentionally small.

### 8.5 PackageBuildDefinition

Extracted from parsed `build.fol`.

Initial required content:

- dependency definitions
- export definitions

Important distinction:

- `build.fol` itself may be full FOL
- phase-one `fol-package` should only extract a narrow, statically recognizable package-definition contract from it
- this phase does not require arbitrary build-code execution

### 8.6 PreparedPackage

This should be the handoff unit from `fol-package` to `fol-resolver`.

It should contain at least:

- `identity`
- optional `metadata`
- optional `build`
- parsed ordinary source package
- canonical source root
- exported root mapping
- dependency identities

Important boundary:

- `PreparedPackage` should carry parsed syntax, not resolved scopes/symbols
- semantic resolution remains resolver work

### 8.7 PackageGraph

Needed for prepared dependency closure:

- entry package
- prepared dependency packages
- identity-indexed cache
- cycle tracking
- export-root mapping

## 9. `package.yaml` Contract

### 9.1 Purpose

- package identity metadata only
- no dependency graph
- no export graph
- no executable behavior

### 9.2 Initial Allowed Fields

- `name`
- `version`
- `kind`
- `description`
- `license`

### 9.3 Rules

- duplicate keys fail
- unknown keys fail
- empty values fail where required
- invalid package names fail
- inline comments remain allowed if the YAML reader already supports them

### 9.4 Explicit Non-Goals

- dependency declarations
- export declarations
- build actions
- source acquisition scripts

## 10. `build.fol` Contract

### 10.1 Purpose

- defines package dependencies
- defines the exported source roots/namespaces for consumers

### 10.2 Current Direction

`build.fol` should be parsed as ordinary FOL through the normal front-end.

Phase one should then recognize package-definition declarations from that parsed build package.

Initial recognized forms stay intentionally narrow:

- `def dep_name: pkg = "...";`
- `def export_name: loc = "...";`

Whether the public spelling stays exactly that or becomes a more explicit nested DSL can be revisited later, but the extracted package contract must stay narrow in phase one.

### 10.3 Rules

- `build.fol` may contain ordinary FOL declarations beyond package-definition declarations
- phase-one package loading must not require executing arbitrary build code
- only statically recognizable top-level package-definition declarations should affect dependency/export loading
- non-string package/export targets are rejected for the recognized package-definition forms
- ordinary declarations that are not recognized package-definition forms must not silently become dependencies or exports
- if later phases want executable build behavior, that should be added as an explicit evaluator/runtime milestone, not implied here

### 10.4 Locator Strategy For Dependencies

The stored dependency target should remain a string at the parser/build-schema boundary.

`fol-package` should later interpret that string as a package locator, for example:

- installed package name/path such as `core`
- namespaced store path such as `org/tools`
- future git locator such as a URL plus revision syntax

The build parser should not hardcode network policy. It should preserve the raw locator cleanly.

## 11. `loc`, `std`, and `pkg` Semantics

### 11.1 `loc`

- takes an exact local directory target
- manifest-free
- build-file-free
- scans `.fol` source files in that directory tree
- loads one directory-backed package surface

### 11.2 `std`

- takes an exact directory target under the configured std root
- manifest-free for now
- build-file-free for now
- behaves like a toolchain-owned local package source

### 11.3 `pkg`

- resolves through `fol-package`
- requires `package.yaml`
- requires `build.fol`
- may later materialize from cache/store/fetch instead of only preinstalled roots
- exposes only what `build.fol` exports

## 12. Source Acquisition Direction For `pkg`

There is no central binary registry goal here.

Desired direction:

- source-based packages
- any git repository can become a package if it has valid root control files
- package installation/materialization happens into a local package store/cache
- resolver consumes already-prepared packages and stays offline/transport-agnostic

That means `fol-package` should eventually own:

- package locator parsing
- fetch/install/update logic
- store layout
- cache keys
- lockfile integration

It must not force that logic into `fol-resolver`.

## 13. C ABI Groundwork

Future C ABI work does not belong in `fol-resolver`.

### 13.1 What Belongs To `fol-package`

- package-level declaration of native artifacts
- package-level declaration of C header or binary link inputs
- package graph attachment of native artifacts to packages

### 13.2 What Does Not Belong Here Yet

- parsing `.h` files
- generating FOL bindings from C headers
- validating C types
- final linker invocation

Those likely belong to later dedicated modules such as:

- a future `fol-c-abi` or `fol-bindgen-c`
- backend/toolchain integration

### 13.3 Groundwork To Reserve Now

The package model should be able to grow from “source-only package” into “package plus artifacts”.

So the model should leave room for future artifact families such as:

- source export
- C header export
- object file input
- static library input
- shared library input

This phase should only reserve the structural slots and explicit unsupported diagnostics if needed. It should not implement C ABI behavior yet.

## 14. Hard Design Rules

### 14.1 No Network In Resolver

- resolver must not fetch, clone, install, or update packages

### 14.2 No Semantic Resolution In `fol-package`

- `fol-package` may parse syntax
- `fol-package` must not build ordinary code symbol tables

### 14.3 Directory-Only Imports Stay Intact

- no file imports as a language concept

### 14.4 Control Files Stay Separate From Ordinary Source

- `package.yaml` and `build.fol` must never be treated as ordinary package source units

### 14.5 Unsupported Package Features Must Fail Explicitly

- unimplemented fetch locators
- unimplemented native artifact kinds
- invalid build forms
- invalid package control roots

## 15. Testing Strategy

The bar should match the lexer/parser/resolver effort: many small explicit tests, not a few broad smoke tests.

### 15.1 `fol-package` Unit Tests

- metadata parser
- metadata validation
- build parser
- build diagnostics
- identity normalization
- directory-root validation
- control-file exclusion
- package-store caching
- cycle detection
- locator parsing

### 15.2 Integration-Style Package Tests

- `loc` local directory load
- `std` load under configured std root
- `pkg` load from installed package store
- exported-root restriction
- transitive `pkg` dependency loading
- shared dependency dedupe
- ignored `package.fol`
- rejected file targets
- malformed `package.yaml`
- malformed `build.fol`

### 15.3 Resolver Integration Tests

- resolver consumes `PreparedPackage` data instead of owning package loading
- ordinary import behavior stays unchanged after the migration
- `loc/std/pkg` still resolve correctly end to end

### 15.4 CLI Integration Tests

- explicit `--std-root`
- explicit `--package-store-root`
- eventual explicit cache/store flags if introduced
- parse-clean and package-invalid programs fail with good diagnostics

### 15.5 C ABI Guardrail Tests

- unsupported native artifact declarations fail explicitly
- package graph can preserve placeholder native-artifact records without pretending they are source exports once that surface exists

## 16. Implementation Phases And Slices

### Phase 0: Reset The Boundary

Status: pending

#### 0.1

Status: done

- Add `fol-package` to the workspace and create the crate shell.

#### 0.2

Status: done

- Define `PackageConfig`, `PackageSourceKind`, `PackageIdentity`, and `PreparedPackage` in `fol-package`.

#### 0.3

Status: done

- Move package metadata parsing out of `fol-resolver` into `fol-package`.

#### 0.4

Status: done

- Move build-definition parsing out of `fol-resolver` into `fol-package`.

### Phase 1: Metadata And Build Schema

Status: pending

#### 1.1

Status: done

- Re-home the existing `package.yaml` parser and tests under `fol-package`.

#### 1.2

Status: done

- Replace the current narrow `build.fol` schema parser with a `fol-package` build extractor over ordinary parsed FOL packages.

#### 1.3

Status: pending

- Add package-loader diagnostics that keep exact origins from `build.fol` parse failures and package-definition extraction failures.

#### 1.4

Status: pending

- Lock explicit tests that `build.fol` is parsed as ordinary FOL, while phase-one package extraction recognizes `def ...: pkg` and `def ...: loc` and ignores or explicitly rejects only unsupported package-definition shapes.

### Phase 2: Package Loading Foundation

Status: pending

#### 2.1

Status: pending

- Build a `PackageSession` in `fol-package` with package cache and loading stack.

#### 2.2

Status: pending

- Move canonical directory/package-root loading logic out of resolver session code.

#### 2.3

Status: pending

- Load `loc` directory packages through `fol-package`.

#### 2.4

Status: pending

- Load `std` directory packages through `fol-package`.

#### 2.5

Status: pending

- Load installed `pkg` roots through `fol-package` with required `package.yaml` + `build.fol`.

### Phase 3: Dependency Graph And Export Graph

Status: pending

#### 3.1

Status: pending

- Move transitive dependency preloading into `fol-package`.

#### 3.2

Status: pending

- Move package-load cycle detection into `fol-package`.

#### 3.3

Status: pending

- Move shared package dedupe into `fol-package`.

#### 3.4

Status: pending

- Compute exported roots/namespaces in `fol-package` and carry them in `PreparedPackage`.

#### 3.5

Status: pending

- Lock tests that control files are excluded from ordinary package source parsing.

### Phase 4: Resolver Integration

Status: pending

#### 4.1

Status: pending

- Make `fol-resolver` depend on `fol-package` for package preparation.

#### 4.2

Status: pending

- Remove resolver-owned metadata/build parsing modules or reduce them to thin adapters slated for deletion.

#### 4.3

Status: pending

- Remove resolver-owned package root loading and caching logic that now belongs in `fol-package`.

#### 4.4

Status: pending

- Keep resolver import semantics stable while changing only the provider boundary.

### Phase 5: CLI And Public API

Status: pending

#### 5.1

Status: pending

- Wire the root CLI through `fol-package` configuration before resolver.

#### 5.2

Status: pending

- Keep `--std-root` and `--package-store-root` working through the new crate boundary.

#### 5.3

Status: pending

- Decide whether a new explicit cache/store flag is needed now or deferred.

### Phase 6: Locator And Distribution Groundwork

Status: pending

#### 6.1

Status: pending

- Introduce a `PackageLocator` model so `build.fol` dependency strings are not treated as opaque forever.

#### 6.2

Status: pending

- Support the current installed-store locators cleanly as one locator family.

#### 6.3

Status: pending

- Add explicit placeholder diagnostics for future git locator forms that are parsed but not yet fetched, or defer parsing them entirely with a documented boundary.

### Phase 7: C ABI Groundwork

Status: pending

#### 7.1

Status: pending

- Add package-model placeholders for future native artifact records without making them active semantics.

#### 7.2

Status: pending

- Document that `.h`, `.o`, static-lib, and shared-lib handling is future package/build work, not resolver work.

### Phase 8: Docs And Closeout

Status: pending

#### 8.1

Status: pending

- Update `README.md`, `PROGRESS.md`, and `FRONTEND_CONTRACT.md` so package loading is described through `fol-package`, not `fol-resolver`.

#### 8.2

Status: pending

- Update book pages covering imports/modules/package control files to the final `fol-package` contract.

#### 8.3

Status: pending

- Rewrite this file into a completion record once `fol-package` owns the package boundary.

## 17. Definition Of Done

This plan is complete when all of the following are true:

- `fol-package` exists as a workspace crate
- package metadata parsing lives in `fol-package`
- build-definition parsing lives in `fol-package`
- package loading/caching/cycle handling lives in `fol-package`
- `fol-resolver` no longer owns package control-file parsing or package-store loading
- `loc/std/pkg` still work end to end
- control files remain excluded from ordinary package source parsing
- resolver behavior is preserved across the provider migration
- docs and tests describe `fol-package` as the package boundary
- the package model has explicit reserved extension points for future C ABI artifacts without pretending that C ABI is already implemented
