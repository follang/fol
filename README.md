<p align="center">
    <img alt="logo" src="./book/src/images/logo.svg" width="300px">
</p>


<a href="https://follang.github.io/" style="color: rgb(179, 128, 255)"></a><h2><p align="center" style="color: rgb(179, 128, 255)">https://follang.github.io/</p></h2></a>

<p align="center">
  <a href="https://github.com/follang/fol/blob/develop/LICENSE.md"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://travis-ci.org/follang/fol"><img alt="Travis (.org)" src="https://img.shields.io/travis/follang/fol"></a>
  <a href="https://codecov.io/github/follang/fol"><img alt="Codecov" src="https://img.shields.io/codecov/c/github/follang/fol"></a>
  <a href="https://gitter.im/follang/community"><img alt="Gitter" src="https://img.shields.io/gitter/room/bresilla/follang"></a>
  <a href="https://github.com/follang/fol/blob/develop/.all-contributorsrc"><img src="https://img.shields.io/badge/all_contributors-1-orange.svg" alt="Contributors"></a>
</p>

<p align="center">general-purpose and systems programming language</p>
<hr>


FOL is a general-purpose, systems programming language designed for robustness, efficiency, portability, expressiveness and most importantly elegance. Heavily inspired (and shamelessly copying) from languages: zig, nim, c++, go, rust, julia (in this order), hence the name - FOL (Frankenstein's Objective Language). In Albanian language "fol" means "speak".

<p align="center">  ** FOL IS AN ACTIVE COMPILER WORKSPACE **  </p>

Current compiler status: `fol-stream`, `fol-lexer`, `fol-parser`,
`fol-package`, `fol-resolver`, `fol-typecheck`, `fol-lower`,
`fol-intrinsics`, `fol-runtime`, `fol-backend`, `fol-frontend`, diagnostics,
and the root CLI are implemented and actively tested. Package loading now
flows through `fol-package`, which prepares directory and installed-package
surfaces ahead of name resolution.
`fol-intrinsics` is now the shared compiler-owned intrinsic registry for the
current `V1` subset, including `.eq(...)`, `.nq(...)`, `.lt(...)`, `.gt(...)`,
`.ge(...)`, `.le(...)`, `.not(...)`, `.len(...)`, `.echo(...)`, `check(...)`,
and `panic(...)`, plus explicit deferred roadmap entries for later milestones.
The diagnostics hardening pass is complete for parser, package loading,
resolver, type checking, lowering, and the CLI: diagnostics carry stable
producer-owned codes, exact primary locations, related labels, notes, helps,
and consistent human/JSON rendering. `fol-typecheck` now covers the full
current `V1` declaration, expression, control-flow, aggregate, shell,
conversion, and unsupported-surface boundary, including workspace-aware `loc` /
`std` / `pkg` imports through the root CLI. `fol-lower` now turns that typed
`V1` workspace into a deterministic backend-facing IR with explicit package
graphs, routines, blocks, exports, entry candidates, and debug snapshots via
`--dump-lowered`. The repaired lowering boundary is now locked end to end for
routine-parameter scoping, typed non-empty container literals, and all-exit
`when` control flow, including one real multi-surface `V1` repro program
exercised through the CLI. Recoverable errors are now part of that current
`V1` contract too: routines declare error types with `ResultType / ErrorType`,
`report expr` is typechecked and lowered, plain errorful calls propagate only
through compatible routine contexts, `check(expr)` and `expr || fallback` are
real handled-call surfaces, and `err[...]` shells remain distinct from routine
call results with declared error types. `fol-runtime` now provides the current
`V1` support layer for strings, containers, shells, recoverable results,
aggregate formatting hooks, `.len(...)`, `.echo(...)`, and top-level
recoverable process outcomes. `fol-backend` now consumes lowered workspaces,
emits deterministic Rust crates against that runtime contract, supports
`--emit-rust` and generated-artifact retention, and can build runnable `V1`
binaries through Cargo. Current end-to-end backend coverage includes
declaration-only inputs, scalar entry programs, records and entries,
containers plus `.len(...)`, `.echo(...)`, recoverable propagation,
`check(...)`, `expr || fallback`, and executable package graphs spanning
`loc`, `std`, and installed `pkg` imports. `fol-frontend` now provides the
user-facing workflow layer on top of that compiler stack: derive-based command
parsing, workspace/package discovery, `init/new/fetch/update/check/build/run/
test/emit/clean/completion`, `work info/list/deps/status`, human/plain/json
output, explicit artifact-root reporting, git-backed dependency fetching with
`fol.lock`, and one canonical `fol` tool.

The next major compiler work no longer sits at the first backend boundary.
That milestone is now real. Follow-on work should build on top of it:
hardening, future backends, `core` / `std`, and later-version language
features.

Current import surface:
- `loc` imports exact filesystem directories
- `std` imports exact directories under an explicit `--std-root`
- `pkg` imports installed package roots under an explicit `--package-store-root`
- no broad cache/store-policy CLI exists yet beyond `fetch`, `update`, and `clean`;
  richer package-manager policy can be added later if the workflow needs it
- `loc` rejects directory targets that already define `build.fol`; formal package
  roots belong to `pkg`
- `pkg` roots require both `package.yaml` and `build.fol`
- `package.yaml` is metadata-only; `build.fol` defines dependency and export records
- stray `package.fol` files are ignored
- consumer-visible `pkg` names come only from build-declared exported roots

Current C ABI boundary:
- `build.fol` may now preserve inert native-artifact placeholders such as `header`,
  `object`, `static_lib`, and `shared_lib`
- `.h`, `.o`, static-library, and shared-library handling is not implemented yet
- native artifact loading, compilation, and linking are future `fol-package` /
  package-build work, not resolver work

For exact current implementation status, treat [`PROGRESS.md`](./PROGRESS.md) as
the repo-backed implementation ledger, treat [`VERSIONS.md`](./VERSIONS.md) as
the V1/V2/V3 language-boundary document, and treat [`PLAN.md`](./PLAN.md) as
the active compiler milestone plan. The README is only a high-level project
summary.

<hr>

## BUILDING BLOCKS


__*Everything*__ in **FOL** is declared like below:

```
	declaration<options> name: returntype = { body; };
	declaration<options> name: returntype = { body; } | { checker } | { alternative; };
```


#### four top-most declarations are:
```
	use    // imports, includes ...
	def    // macros, bocks, definitions ...

	var    // all variables: ordinal, container, complex, special

	pro    // subporgrams with side effects - procedures
	fun    // subporgrams with no side effects - functions
	log    // subporgrams with logic only - logicals

	typ    // new types: records, entries, blueprints ...
	ali    // aiased types and extensions
```
#### a control flow and keywords:
```
	when(condition){ case (){}; case (){}; * {}; };
	loop(condition){  };

```

#### example:

```
use log: std = {"fmt/log"};


def argo: mod[init] = {
    -var const: str = "something here"

    +pro main: int = {
        log.warn("Last warning!...");
        .echo(add(3, 5));
    }

    fun add(a, b: int): int = { a + b }
}
```
