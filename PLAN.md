# Git Dependency Selector Cutover Plan

This plan replaces the current git dependency selector surface:

- selectors embedded inside `target`
- examples like `git+https://github.com/org/repo.git?branch=main`
- optional `?tag=...`
- optional `?rev=...`
- optional `?hash=...`

with a structured build API:

```fol
pro[] build(): non = {
    var build = .build();

    build.add_dep({
        alias = "logtiny",
        source = "git",
        target = "git+https://github.com/bresilla/logtiny.git",
        version = "tag:v0.1.1",
        hash = "77df4240d6f0",
    });
}
```

The goal is:

1. keep `target` as only the repository locator
2. move selector policy into explicit fields
3. keep hash verification explicit and separate
4. remove query-param selectors entirely
5. verify the real flow against `xtra/logtiny`

This follows the repo policy:

- no legacy shims
- no compatibility parsing path
- no dual public syntax

If the new structured surface is chosen, the old `?branch=...`, `?tag=...`,
`?rev=...`, and `?hash=...` form must be deleted, not deprecated.

## Current Grounding

The current implementation that this plan will replace lives in:

- locator parsing:
  [lang/compiler/fol-package/src/locator.rs](/home/bresilla/data/code/bresilla/fol/lang/compiler/fol-package/src/locator.rs)
- git revision resolution and materialization:
  [lang/compiler/fol-package/src/git.rs](/home/bresilla/data/code/bresilla/fol/lang/compiler/fol-package/src/git.rs)
- package metadata extraction from `build.fol`:
  [lang/compiler/fol-package/src/metadata.rs](/home/bresilla/data/code/bresilla/fol/lang/compiler/fol-package/src/metadata.rs)
- build evaluator dependency request parsing:
  [lang/execution/fol-build/src/executor/handle_methods.rs](/home/bresilla/data/code/bresilla/fol/lang/execution/fol-build/src/executor/handle_methods.rs)
  [lang/execution/fol-build/src/executor/resolve.rs](/home/bresilla/data/code/bresilla/fol/lang/execution/fol-build/src/executor/resolve.rs)
- dependency request types:
  [lang/execution/fol-build/src/api/types.rs](/home/bresilla/data/code/bresilla/fol/lang/execution/fol-build/src/api/types.rs)
- frontend fetch / lockfile behavior:
  [lang/tooling/fol-frontend/src/fetch.rs](/home/bresilla/data/code/bresilla/fol/lang/tooling/fol-frontend/src/fetch.rs)
  [lang/compiler/fol-package/src/lockfile.rs](/home/bresilla/data/code/bresilla/fol/lang/compiler/fol-package/src/lockfile.rs)
- current examples/tests:
  [examples/std_logtiny_git](/home/bresilla/data/code/bresilla/fol/examples/std_logtiny_git)
  [xtra/logtiny](/home/bresilla/data/code/bresilla/fol/xtra/logtiny)
  [test/integration_tests/integration_editor_and_build.rs](/home/bresilla/data/code/bresilla/fol/test/integration_tests/integration_editor_and_build.rs)

## New Public Contract

For git dependencies:

- `target` must be only the repository locator
- `version` is optional
- `hash` is optional

Allowed forms:

```fol
build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
});
```

```fol
build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
    version = "branch:develop",
});
```

```fol
build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
    version = "tag:v0.1.1",
});
```

```fol
build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
    version = "commit:77df4240d6f0a28590fc5b8dce8b648b63c17540",
});
```

```fol
build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
    version = "branch:develop",
    hash = "77df4240d6f0",
});
```

Semantic rules:

- `version` accepted schemes:
  - `branch:<name>`
  - `tag:<name>`
  - `commit:<sha>`
- `hash` means required commit-prefix verification
- `version` absent means remote `HEAD`
- `hash` may be used with or without `version`

Rejected:

- selectors inside `target`
- `version = "rev:..."`
- multiple version selectors
- empty `version`
- empty `hash`

## Epoch 1: Freeze The New Contract

### Slice 1
Status: complete

- document the new public contract in the build book
- explicitly state:
  - `target` is repository only
  - `version` carries selection
  - `hash` carries verification
- explicitly state old query-param selectors are removed

### Slice 2
Status: complete

- update standalone examples in docs to the new structured shape
- remove any public-facing `?branch=`, `?tag=`, `?rev=`, `?hash=` examples

### Slice 3
Status: complete

- add one short architecture note to `PLAN.md` / book direction docs:
  - structured dependency config is preferred over query-param encoding

## Epoch 2: Add Structured Dependency Fields

### Slice 4
Status: complete

- extend dependency config parsing in the build evaluator to accept:
  - `version`
  - `hash`
- only for `source = "git"`

### Slice 5
Status: complete

- extend dependency request types so git dependencies carry structured selector
  data instead of only an opaque target string

### Slice 6
Status: complete

- define an internal selector model:
  - none
  - branch
  - tag
  - commit
- define separate optional verification hash

### Slice 7
Status: complete

- add semantic registry coverage for the new config fields
- ensure build-editor completion/help sees `version` and `hash`

## Epoch 3: Delete Query-Param Selector Parsing

### Slice 8
Status: complete

- remove `branch`, `tag`, `rev`, `hash` parsing from git locator query params
- keep repository locator parsing only

### Slice 9
Status: complete

- simplify `PackageGitSelector` / locator model to reflect the new split
- locator should no longer be responsible for public selector parsing

### Slice 10
Status: complete

- delete tests that assert query-param selectors parse
- replace them with tests that query-param selectors are rejected

### Slice 11
Status: complete

- make the rejection diagnostic explicit:
  - selector query params are no longer supported
  - use `version` and `hash` fields on `build.add_dep(...)`

## Epoch 4: Version Field Parsing And Validation

### Slice 12
Status: complete

- implement parser/validator for:
  - `branch:<name>`
  - `tag:<name>`
  - `commit:<sha>`

### Slice 13
Status: complete

- reject malformed `version` values with exact diagnostics:
  - missing `:`
  - unknown selector kind
  - empty branch/tag/commit payload

### Slice 14
Status: complete

- reject `version` on non-git dependencies with exact diagnostics

### Slice 15
Status: complete

- reject `hash` on non-git dependencies with exact diagnostics

### Slice 16
Status: complete

- reject empty-string `hash`

## Epoch 5: Wire Structured Selectors Into Git Resolution

### Slice 17
Status: complete

- change git revision resolution to use structured selector data from dependency
  requests instead of query params embedded in locators

### Slice 18
Status: complete

- map `version = "branch:..."` to remote branch resolution

### Slice 19
Status: complete

- map `version = "tag:..."` to remote tag resolution

### Slice 20
Status: complete

- map `version = "commit:..."` to direct commit pinning

### Slice 21
Status: complete

- keep `hash` verification as resolved-commit prefix checking

### Slice 22
Status: complete

- make the mismatch diagnostic exact and stable:
  - include dependency locator
  - include resolved revision
  - include required hash

## Epoch 6: Metadata, Fetch, And Lockfile Integration

### Slice 23
Status: complete

- change package metadata extraction from `build.fol` so it captures:
  - repo locator
  - version selector
  - hash
- do not keep selector info encoded in `target`

### Slice 24
Status: complete

- update frontend fetch resolution to use structured selector fields

### Slice 25
Status: complete

- ensure eager/lazy/on-demand behavior remains unchanged under the new model

### Slice 26
Status: complete

- update lockfile rendering/parsing if needed so stored entries remain sufficient
  to reproduce exact pinned git state

### Slice 27
Status: complete

- decide and implement whether lockfile should additionally record requested
  `hash` or only selected revision
- preferred direction:
  - keep selected revision authoritative
  - optionally keep requested hash for transparency if useful

## Epoch 7: Tests For The New Public Surface

### Slice 28
Status: complete

- add evaluator tests for:
  - plain repo target with no version
  - branch version
  - tag version
  - commit version
  - branch plus hash

### Slice 29
Status: complete

- add evaluator tests for invalid config:
  - bad `version`
  - bad `hash`
  - `version` on `pkg`
  - `hash` on `loc`

### Slice 30
Status: complete

- add package metadata extraction tests for the new fields

### Slice 31
Status: complete

- add frontend fetch tests that materialize local temp git repos using:
  - branch
  - tag
  - commit
  - branch plus hash

### Slice 32
Status: complete

- add one explicit negative fetch test for hash mismatch

## Epoch 8: Real `xtra/logtiny` Verification

### Slice 33

- update [xtra/logtiny/build.fol](/home/bresilla/data/code/bresilla/fol/xtra/logtiny/build.fol)
  if needed so it remains a valid dependency target:
  - metadata
  - exported module
  - optional exported artifact

### Slice 34

- verify `xtra/logtiny` directly with the local CLI:
  - `code build`
  - built binary execution if applicable

### Slice 35

- commit `xtra/logtiny` changes in its own repo with conventional commit title

### Slice 36

- push the branch to its remote

### Slice 37

- add and push a real tag for verification

### Slice 38

- add integration coverage in this repo using the live GitHub `logtiny` repo for:
  - branch
  - tag
  - commit
  - branch plus hash

## Epoch 9: Migrate Examples And Remove Old Surface

### Slice 39

- migrate [examples/std_logtiny_git/build.fol](/home/bresilla/data/code/bresilla/fol/examples/std_logtiny_git/build.fol)
  to the new shape

### Slice 40

- migrate any other examples that still use selector query params

### Slice 41

- update integration fixtures and helper-generated build files to the new
  structured fields

### Slice 42

- remove all checked-in public examples of query-param selectors

## Epoch 10: Hardening And Final Cleanup

### Slice 43

- audit editor/LSP build completion tests for `version` and `hash`

### Slice 44

- ensure diagnostics point users only to the new shape

### Slice 45

- update build book chapters:
  - dependency config
  - git examples
  - verification examples

### Slice 46

- add one standalone example package specifically for:
  - git branch pin
  - git tag pin
  - git commit pin
  - git hash verification

### Slice 47

- remove stale code/comments/tests that still assume selector query params

### Slice 48

- final full pass:
  - `make build`
  - `make test`
  - worktree clean

## Expected Outcome

At the end of this plan:

- git repo locators are clean and selector-free
- selector policy lives in explicit build fields
- hash verification is explicit and first-class
- old query-param selectors are gone
- docs/examples/tests all teach only the new structured form
- the flow is verified against the real `xtra/logtiny` dependency repo
