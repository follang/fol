# PLAN: Split Capability Modes From Bundled `std`

This plan migrates FOL from the current three-way public model:

- `core`
- `mem`
- `std`

to the finalized design:

- `fol_model = "core"`
- `fol_model = "memo"`
- omitted `fol_model` means `memo`
- bundled standard library is an explicit dependency declared in `build.fol`
- the bundled package identity is `standard`
- the normal dependency alias in user projects is `std`
- `core` and `memo` are capability modes only, not importable libraries
- only `std` is importable in source code, and only when the bundled `standard`
  dependency was declared

This plan intentionally removes the old public `fol_model = "std"` story.

It also removes the special “std is just globally mounted because the artifact is
in std mode” mental model.

The resulting design should be:

- capability selection is one concept
- dependency declaration is another concept
- source imports follow the dependency system
- bundled std ships with FOL, but still enters projects explicitly through
  `.build().add_dep(...)`

The default user story should become:

```fol
pro[] build(): non = {
    var build = .build();

    build.meta({
        name = "app",
        version = "0.1.0",
    });

    build.add_dep({
        alias = "std",
        source = "internal",
        target = "standard",
    });

    var graph = build.graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
        fol_model = "memo",
    });

    graph.install(app);
    graph.add_run(app);
};
```

and then in source:

```fol
use io: pkg = {std/io};
```

or whatever equivalent `pkg` import spelling the compiler currently accepts for
dependency-rooted package paths.

This plan does **not**:

- invent importable `core` or `memo` libraries
- keep compatibility for `fol_model = "std"`
- keep compatibility for `fol_model = "mem"`
- keep globally mounted bundled std in normal resolution
- preserve special std-only import semantics if the dependency model replaces them
- add migration shims, fallback parsing, or dual behavior

---

## Design Freeze

The end state must satisfy all of these:

- `fol_model` public spellings are only:
  - `core`
  - `memo`
- omitted `fol_model` defaults to `memo`
- `core` means:
  - no heap-backed families
  - no bundled std import
- `memo` means:
  - heap-backed families allowed
  - bundled std import still requires explicit dependency declaration
- bundled std is added through:
  - `source = "internal"`
  - `target = "standard"`
- projects normally bind that dependency as:
  - `alias = "std"`
- source code reaches bundled std through the dependency/import mechanism, not
  through a special always-on std mount
- `graph.add_run(...)` remains independent from bundled std presence
- bundled std remains shipped with FOL under:
  - [lang/library/std](/home/bresilla/data/code/bresilla/fol/lang/library/std)
- `--std-root` may remain as a dev/test override only if still useful for the
  internal-provider path; it must not remain the normal explanation

Open implementation seam that the plan must settle explicitly:

- whether the source-level import kind `std` survives as syntax sugar over the
  bundled internal dependency, or whether bundled std becomes ordinary `pkg`
  import only

Preferred outcome for this plan:

- bundled std becomes ordinary dependency-backed import
- docs and examples use `pkg`-style dependency import rooted at alias `std`
- special `use ...: std = ...` handling is removed

If the codebase proves that deleting the special `std` import kind cleanly in the
same migration is too wide, this plan still requires:

- the dependency declaration becomes mandatory
- normal semantic ownership moves to the dependency system
- any remaining `std` import syntax must be clearly documented as thin sugar over
  the bundled dependency, not as a separate conceptual model

---

## Epoch 1: Freeze The New Model

### Slice 1
Status: pending

Write the finalized design contract into active docs before code movement.

Completion criteria:

- one active doc states:
  - `core` and `memo` are the only public capability modes
  - `standard` is the bundled package identity
  - `std` is the normal dependency alias
- docs stop describing `std` as a third `fol_model`

### Slice 2
Status: pending

Add a top-level contract matrix test for the new split.

Completion criteria:

- one test locks:
  - `core` rejects heap-backed families
  - `memo` accepts heap-backed families
  - bundled std import requires dependency declaration
  - run/build steps are not tied to std dependency presence

### Slice 3
Status: complete

Audit active docs/examples/tests for the exact public terms:

- `mem`
- `std` as a model
- “std is globally available by mode”

Completion criteria:

- execution notes list all public seams that still encode the old model

### Slice 4
Status: pending

Freeze the import-side target syntax for bundled std.

Completion criteria:

- one doc/test pins the intended source-side spelling
- if `pkg` is the target, at least one example shows it explicitly

---

## Epoch 2: Rename `mem` To `memo`

### Slice 5
Status: complete

Change public `fol_model` validation from `mem` to `memo`.

Completion criteria:

- `memo` is accepted
- `mem` is rejected publicly

### Slice 6
Status: complete

Thread `memo` through build-eval model parsing and defaulting.

Completion criteria:

- omitted `fol_model` resolves to `memo`
- explicit `core` and `memo` both work

### Slice 7
Status: complete

Update typecheck capability naming to `memo`.

Completion criteria:

- diagnostics refer to `memo`
- public capability matrix tests use `memo`

### Slice 8
Status: complete

Update frontend/build-summary/reporting paths to `memo`.

Completion criteria:

- summaries and diagnostics no longer say public `mem`

### Slice 9
Status: complete

Update editor/LSP/tree-sitter public model-facing fixtures to `memo`.

Completion criteria:

- model tokens, completion, hover, and fixtures use `memo`

### Slice 10
Status: complete

Migrate active docs/examples from `mem` to `memo`.

Completion criteria:

- public examples no longer spell `mem`
- docs/book no longer spell `mem` as the current public model

---

## Epoch 3: Make `memo` The Default

### Slice 11
Status: complete

Add or tighten build-eval semantics for omitted `fol_model`.

Completion criteria:

- omitted `fol_model` defaults to `memo`
- tests prove the default

### Slice 12
Status: complete

Update graph artifact docs/examples so explicit `fol_model` is no longer required
for ordinary heap-backed hosted projects.

Completion criteria:

- at least one example intentionally omits `fol_model`
- the resulting artifact reports `memo`

### Slice 13
Status: pending

Add diagnostics for stale `fol_model = "std"` and `fol_model = "mem"`.

Completion criteria:

- both old public spellings fail
- diagnostics point to `core` and `memo`

### Slice 14
Status: complete

Add scaffold/init coverage for the new default.

Completion criteria:

- scaffolded ordinary app builds omit `fol_model` or spell `memo`
- no scaffold emits `mem` or `std` as public model spellings

---

## Epoch 4: Introduce Bundled Internal Dependency Source

### Slice 15
Status: pending

Define the public build API for internal shipped dependencies.

Completion criteria:

- `build.add_dep({...})` accepts:
  - `source = "internal"`
  - `target = "standard"`
- docs describe this as the shipped dependency source kind

### Slice 16
Status: pending

Add build-eval validation for internal dependency configs.

Completion criteria:

- valid `internal/standard` config succeeds
- bad `internal` target shapes fail clearly

### Slice 17
Status: pending

Thread internal dependency requests through package metadata extraction.

Completion criteria:

- prepared package metadata preserves:
  - `source = "internal"`
  - `target = "standard"`

### Slice 18
Status: pending

Teach dependency materialization/session code how to resolve bundled internal deps.

Completion criteria:

- `internal:standard` resolves to the shipped bundled std root
- no network/package-store lookup is attempted for it

### Slice 19
Status: pending

Add fetch/lockfile/frontend behavior for internal deps.

Completion criteria:

- internal deps are represented honestly in summaries/lock behavior
- no fake git/pkg locator is used

### Slice 20
Status: pending

Add a standalone example that declares bundled std explicitly in `build.fol`.

Completion criteria:

- one example adds:
  - `alias = "std"`
  - `source = "internal"`
  - `target = "standard"`
- it builds and runs

---

## Epoch 5: Move Bundled `std` Onto The Dependency System

### Slice 21
Status: pending

Stop globally mounting bundled std during normal resolution.

Completion criteria:

- normal resolution does not expose std unless declared as a dependency

### Slice 22
Status: pending

Wire bundled std visibility through declared dependency aliases.

Completion criteria:

- alias `std` resolves bundled std modules in source
- a different alias also works if declared intentionally

### Slice 23
Status: pending

Add negative coverage for missing bundled std dependency declarations.

Completion criteria:

- code importing std without declaring internal `standard` dependency fails clearly

### Slice 24
Status: pending

Add negative coverage for declaring bundled std under one alias and importing under
another.

Completion criteria:

- dependency alias mismatch fails clearly

### Slice 25
Status: pending

Update resolver/package docs to explain bundled std as dependency-backed.

Completion criteria:

- active docs stop saying std is always present because of the model alone

---

## Epoch 6: Replace `std` As A Model With `std` As A Dependency

### Slice 26
Status: pending

Remove public `fol_model = "std"` acceptance everywhere.

Completion criteria:

- build evaluation rejects `std` as a model spelling
- frontend and editor diagnostics reject it too

### Slice 27
Status: pending

Rewrite runtime-model docs around:

- `core`
- `memo`
- explicit bundled std dependency

Completion criteria:

- no active doc still presents `std` as a third model

### Slice 28
Status: pending

Update integration examples that currently use `fol_model = "std"`.

Completion criteria:

- hosted examples instead use:
  - `fol_model = "memo"` or omitted model
  - explicit internal `standard` dependency

### Slice 29
Status: pending

Update routed workspace/model reporting to reflect the new split.

Completion criteria:

- reporting distinguishes capability mode from dependency presence
- mixed-workspace reports no longer describe std as a model

### Slice 30
Status: pending

Add a matrix test covering:

- `core` without std
- `memo` without std
- `memo` with std

Completion criteria:

- each case is asserted across build/typecheck behavior

---

## Epoch 7: Settle Source Import Semantics

### Slice 31
Status: pending

Choose whether bundled std continues to use special `std` import syntax.

Completion criteria:

- one implementation direction is chosen and documented

### Slice 32
Status: pending

If the chosen direction is dependency-backed `pkg` only, delete the special `std`
source kind.

Completion criteria:

- source imports reach bundled std through dependency-rooted import
- old special std import path is removed

### Slice 33
Status: pending

If temporary std import sugar remains, make it resolve strictly through declared
internal dependency aliases.

Completion criteria:

- sugar no longer means “std is ambient”
- tests prove it depends on declared bundled std dependency presence

### Slice 34
Status: pending

Update parser/resolver/editor tests for the chosen import contract.

Completion criteria:

- completion/hover/definition tests reflect the final import form

### Slice 35
Status: pending

Update all active examples to the final import form.

Completion criteria:

- no active example uses the wrong bundled std import style

---

## Epoch 8: Harden Runtime And Build Semantics

### Slice 36
Status: pending

Add explicit tests proving `graph.add_run(...)` is independent from std dependency.

Completion criteria:

- runnable `core` or `memo` artifacts without std are covered where honest
- diagnostics only fail for actual language/runtime restrictions, not for lack of std

### Slice 37
Status: pending

Add tests proving hosted std wrappers still require bundled std dependency presence.

Completion criteria:

- `std.io` usage without the internal `standard` dependency fails

### Slice 38
Status: pending

Audit backend/runtime import selection under the new split.

Completion criteria:

- backend emission tracks capability mode independently from std dependency
- generated Rust import tests remain honest

### Slice 39
Status: pending

Tighten build summaries and `work info` to surface:

- capability mode
- bundled std dependency presence

Completion criteria:

- summaries help users understand why std imports are or are not available

### Slice 40
Status: pending

Add mixed-workspace regression coverage:

- some packages with bundled std dependency
- some without
- mixed `core` and `memo`

Completion criteria:

- editor/frontend routing behaves correctly in mixed graphs

---

## Epoch 9: Update Bundled `std` Itself

### Slice 41
Status: pending

Rewrite [lang/library/std/README.md](/home/bresilla/data/code/bresilla/fol/lang/library/std/README.md)
for the new dependency model.

Completion criteria:

- README says std is shipped with FOL
- README says projects still declare it explicitly as internal dependency

### Slice 42
Status: pending

Update bundled std bootstrap examples to the new explicit-dependency model.

Completion criteria:

- `std_bundled_fmt`
- `std_bundled_io`
- both declare internal `standard` dependency explicitly

### Slice 43
Status: pending

Add one negative example for missing bundled std dependency.

Completion criteria:

- checked-in example fails because `std` is imported without adding internal `standard`

### Slice 44
Status: pending

Add one alias-variation example showing that bundled std is just a dependency root.

Completion criteria:

- one example intentionally binds bundled std under a non-`std` alias
- source imports follow that alias correctly

---

## Epoch 10: Final Cleanup And Closure

### Slice 45
Status: pending

Update book chapters to remove the old “`std` is a model” explanation.

Completion criteria:

- build/import book chapters present:
  - `core`
  - `memo`
  - internal `standard` dependency

### Slice 46
Status: pending

Update contributor docs and AGENTS guidance.

Completion criteria:

- contributor docs say:
  - `core` and `memo` are capability modes only
  - bundled std is explicit internal dependency

### Slice 47
Status: pending

Run a repo-wide stale wording sweep for:

- `fol_model = "std"`
- public `mem`
- “std is globally mounted by mode”

Completion criteria:

- active docs/examples/tests contain no stale public wording

### Slice 48
Status: pending

Add a final contract integration test.

Completion criteria:

- one top-level integration test covers the settled end-to-end story:
  - default `memo`
  - explicit bundled std dependency
  - correct import behavior
  - missing dependency failure

### Slice 49
Status: pending

Audit the `xtra/logtiny` style external-dependency examples against the new std
story.

Completion criteria:

- examples/docs clearly distinguish:
  - external deps
  - bundled internal `standard`

### Slice 50
Status: pending

Close the plan with an honesty audit.

Completion criteria:

- active docs/tests/examples do not claim more than the implementation really
  supports
- `PLAN.md` is fully marked complete only after build/test stay green
