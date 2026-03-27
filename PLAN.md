# PLAN: Finalize Capability Modes And Bundled Std

This plan freezes the final architecture for capability modes and bundled
standard-library access.

There is no compatibility path.

If the repo still contains behavior or wording from older models, this plan
deletes it.

## Final Contract

The target contract is:

- public capability modes are only:
  - `core`
  - `memo`
- omitted `fol_model` defaults to `memo`
- `std` is not a capability mode
- bundled std is only available through an explicit internal dependency:
  - `source = "internal"`
  - `target = "standard"`
- normal projects are expected to bind bundled std as:
  - `alias = "std"`
- source code may only use bundled std after that dependency exists
- the normal source-side import form is:
  - `use std: pkg = {"std"};`
- `core` and `memo` are not importable libraries
- `graph.add_run(app)` is independent of std-library presence
- direct hosted substrate like `.echo(...)` remains temporarily available as a
  low-level primitive, but docs/examples should increasingly prefer bundled
  `std::io`

## Why

This separates three concerns cleanly:

- capability policy
- bundled standard library availability
- runnable artifact orchestration

That means:

- `fol_model` controls language/runtime legality
- internal bundled dependency controls std-library availability
- build graph run/install behavior is not secretly tied to std imports

This avoids the older ambiguity where `std` tried to mean all of:

- hosted runtime
- standard library
- runnability
- import source kind

## Non-Goals

This plan does not:

- make `core` importable
- make `memo` importable
- add a public `std` source kind back
- keep `fol_model = "std"`
- add compatibility shims
- add migration warnings
- fully remove direct `.echo(...)` in the first pass

## Architectural Rules

### Capability modes

Only these public mode spellings remain valid:

- `core`
- `memo`

Meaning:

- `core`
  - no heap-backed surfaces
  - no bundled std unless explicitly allowed later by policy
- `memo`
  - heap-backed surfaces allowed
  - bundled std still requires explicit dependency

### Defaulting

If an artifact omits `fol_model`, it should behave as:

- `fol_model = "memo"`

This default must be consistent in:

- build evaluation
- frontend summaries
- routed planning
- docs/examples/scaffolding
- editor/LSP model reporting

### Bundled std

Bundled std is a shipped internal dependency only.

The canonical build declaration is:

```fol
build.add_dep({
    alias = "std",
    source = "internal",
    target = "standard",
});
```

The canonical source import is:

```fol
use std: pkg = {"std"};
```

### Import legality

If bundled std is not declared, source code must not be able to use:

```fol
use std: pkg = {"std"};
```

or any alias-bound equivalent path.

That must fail clearly in:

- resolver
- CLI
- editor/LSP

### Runnability

This plan keeps runnability separate from std-library presence.

That means:

- `graph.add_run(app)` does not require bundled std
- runnable `core` and `memo` artifacts are still allowed if otherwise valid
- std-library use is about declared library availability, not whether an
  artifact can be run

### Hosted substrate

`.echo(...)` remains temporarily available as low-level hosted substrate.

Practical rule:

- do not break working hosted primitives in this architecture pass
- do shift user-facing docs/examples toward bundled `std::io`
- keep the wording honest that `std::io` is the preferred public wrapper and
  `.echo(...)` is substrate-level for now

## Epoch 1: Freeze The Contract

### Slice 1
Status: completed

Write the final contract into active top-level docs:

- `fol_model = core | memo`
- default `memo`
- std is explicit internal dependency
- std is imported through `pkg`
- `graph.add_run` is independent of std

### Slice 2
Status: completed

Write the same contract into contributor-facing docs:

- `AGENTS.md`
- any active architecture notes under `docs/`

### Slice 3
Status: completed

Add one compiler/package/frontend contract matrix test that pins:

- `core` valid
- `memo` valid
- omitted mode becomes `memo`
- `std` mode invalid

### Slice 4
Status: completed

Add one contract test that pins bundled std as:

- `source = "internal"`
- `target = "standard"`

and rejects alternative internal target spellings.

## Epoch 2: Delete `std` As A Mode

### Slice 5
Status: completed

Remove `std` from public `fol_model` parsing/validation.

### Slice 6
Status: completed

Update build-eval diagnostics so `fol_model = "std"` fails with exact contract
wording.

### Slice 7
Status: completed

Update typecheck/model-capability tests so public model matrices only mention:

- `core`
- `memo`

### Slice 8
Status: completed

Update frontend/routed planning summaries so they no longer surface `std` as a
mode.

### Slice 9
Status: completed

Update editor/LSP model recovery and reporting so `std` is no longer treated as
an artifact mode.

## Epoch 3: Default To `memo`

### Slice 10
Status: completed

Make omitted artifact `fol_model` default to `memo` everywhere the build graph
is evaluated.

### Slice 11
Status: completed

Add integration coverage showing omitted `fol_model` builds as `memo`.

### Slice 12
Status: completed

Update scaffold/init/default example outputs to omit explicit `memo` where the
default is intended, or keep it only if the chosen docs style wants explicit
mode spelling. Pick one and apply it consistently.

### Slice 13
Status: completed

Update summaries/docs so the default is stated concretely and not implied
through older `std` wording.

## Epoch 4: Explicit Bundled Std Dependency Semantics

### Slice 14
Status: completed

Audit `build.add_dep({ source = "internal", target = "standard" })` through:

- build eval
- metadata extraction
- package session preparation
- resolver session loading

and pin it as the only bundled-std acquisition path.

### Slice 15
Status: completed

Add dependency tests showing the normal alias is:

- `std`

but other aliases technically work if deliberately chosen.

### Slice 16
Status: completed

Add resolver tests proving `use std: pkg = {"std"};` fails when the dependency
was not declared.

### Slice 17
Status: completed

Add resolver tests proving alias mismatch fails cleanly:

- declared alias is not `std`
- source still imports `{"std"}`

### Slice 18
Status: completed

Add integration coverage for bundled std through:

- build
- check
- run
- dump lowered

using the explicit dependency only.

## Epoch 5: Runnability Independence

### Slice 19
Status: completed

Audit build/frontend/run behavior so `graph.add_run(app)` no longer relies on a
former `std` mode assumption.

### Slice 20
Status: completed

Add one positive `core` runnable example/fixture with no std dependency.

### Slice 21
Status: completed

Add one positive `memo` runnable example/fixture with no std dependency.

### Slice 22
Status: completed

Add integration tests proving:

- runnable `core` artifact without std dependency
- runnable `memo` artifact without std dependency
- std dependency presence is orthogonal to run-step legality

### Slice 23
Status: completed

Update docs so they stop implying “hosted run requires std dependency” unless
that is explicitly intended somewhere else.

## Epoch 6: Hosted Substrate Versus Public Std

### Slice 24
Status: completed

Freeze the wording around `.echo(...)`:

- still allowed as substrate
- not the preferred public-library story
- `std.io` is the preferred wrapper

### Slice 25
Status: completed

Add tests/docs showing:

- `memo` without std dependency can still use substrate-hosted behavior if the
  language currently allows it
- bundled `std.io` is a library wrapper, not the only path to hosted output

### Slice 26
Status: completed

Convert active public examples that are meant to demonstrate standard-library
use so they prefer `std::io`.

### Slice 27
Status: completed

Keep one intentionally small example that still demonstrates raw `.echo(...)`
as substrate, and label it honestly.

## Epoch 7: Editor And Tree-sitter Contract

### Slice 28
Status: completed

Update LSP tests so capability reporting speaks only in `core` and `memo`, with
bundled std handled as dependency presence.

### Slice 29
Status: completed

Add editor diagnostics proving:

- `use std: pkg = {"std"};` without declared dependency fails in editor path
- parser success but resolver failure is surfaced correctly

### Slice 30
Status: completed

Update tree-sitter/example coverage if any current corpus or fixture still
describes `std` as a source kind or mode.

### Slice 31
Status: completed

Add one editor integration regression for omitted `fol_model` defaulting to
`memo`.

## Epoch 8: Examples And Fixtures

### Slice 32
Status: completed

Audit all checked-in examples so they follow one of these patterns only:

- `core` with no std dependency
- `memo` with no std dependency
- `memo` plus explicit bundled std dependency

### Slice 33
Status: completed

Add one minimal canonical example for:

- `core` runnable without std
- `memo` runnable without std
- `memo` + bundled std dependency

### Slice 34
Status: completed

Add one negative example:

- `use std: pkg = {"std"};`
- missing explicit internal std dependency

### Slice 35
Status: completed

Update app fixtures and formal fixtures so none of them still encode the older
`std`-mode story.

## Epoch 9: Docs And Book Alignment

### Slice 36
Status: completed

Rewrite runtime-model docs so they describe:

- `core`
- `memo`
- bundled std dependency

instead of `core/memo/std` as peer modes.

### Slice 37
Status: completed

Rewrite import docs so bundled std is taught only through:

- explicit internal dependency in `build.fol`
- `use std: pkg = {"std"};`

### Slice 38
Status: completed

Rewrite build docs so they clearly separate:

- capability mode selection
- bundled std dependency declaration
- run/install graph setup

### Slice 39
Status: completed

Rewrite book examples that still imply the removed `std` mode.

### Slice 40
Status: completed

Update contributor guidance so future changes do not reintroduce:

- `fol_model = "std"`
- ambient std assumptions
- `std` as a source kind

## Epoch 10: Final Cleanup And Closure

### Slice 41
Status: completed

Repo-wide stale sweep for:

- `fol_model = "std"`
- “std mode”
- `: std =`
- docs that tie runnability to std presence

### Slice 42
Status: completed

Add one top-level sync test scanning docs/examples/fixtures for stale removed
contracts:

- `fol_model = "std"`
- `: std =`

### Slice 43
Status: completed

Add one top-level sync test scanning docs/examples/fixtures for the required
bundled-std contract:

- `source = "internal"`
- `target = "standard"`
- `use std: pkg = {"std"};`

### Slice 44
Status: completed

Run targeted negative integration checks for:

- missing std dependency
- old `std` mode spelling
- bad internal target
- alias mismatch

### Slice 45
Status: completed

Run the full repo gate:

- `make build`
- `make test`

Only after that, mark the plan complete.
