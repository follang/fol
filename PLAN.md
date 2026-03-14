# Full V1 Typecheck Completion Record

Status: done

## 1. Outcome

- `fol-typecheck` now covers the full current `V1` semantic boundary for the
  implemented language surface.
- The root CLI now runs the full:
- `fol-package -> fol-resolver -> fol-typecheck`
  chain against resolver workspaces instead of the legacy single-package path.
- Imported `loc`, `std`, and installed `pkg` symbols now keep declaration and
  expression typing parity with local symbols.
- Imported method lookup now uses typed foreign-package facts instead of
  entry-package syntax scans.
- Imported record, entry, optional, and error-shell surfaces now participate in
  the current `V1` typecheck contract wherever their local equivalents already
  do.
- The legacy single-package typecheck API remains as a compatibility surface and
  still rejects imported package graphs explicitly; the full compiler path is
  workspace-aware.

## 2. Validation Baseline

- `make build`: passed
- `make test`: passed
- Observed current totals:
- `8` unit tests passed
- `1503` integration tests passed

## 3. What This Milestone Closed

- Graph-aware resolver handoff for semantic consumers.
- Mounted-symbol provenance across loaded packages.
- Graph-aware declaration signature lowering.
- Imported value, routine, type, call, and method typing parity.
- Named aggregate expansion parity across imported packages.
- Current `V1` optional / error-shell typing, including `nil` and postfix
  unwrap.
- Removal of user-triggerable raw lowered-type / internal-style fallback errors
  from valid parsed+resolved `V1` programs.
- Exact diagnostics for reopened surfaces, including CLI JSON coverage.
- End-to-end CLI parity for direct folder entries, `loc`, `std`, and installed
  `pkg` imports.

## 4. Definition Of Done Met

- Imported symbols that resolve cleanly now typecheck cleanly through the full
  compiler chain.
- Imported mismatches now fail as ordinary `Typecheck*` diagnostics instead of
  reopened blocker placeholders.
- Valid parsed+resolved `V1` programs no longer fail because the compiler stayed
  on the legacy single-package semantic path.
- Current repo docs now describe `fol-typecheck` as the full current `V1`
  typechecker.

## 5. Next Boundary

- Stay inside `V1` for the next compiler stages:
- later semantic lowering
- backend / runtime / binary-producing direction
- Treat [`VERSIONS.md`](./VERSIONS.md) as the language-boundary reference for
  what remains in `V2` and `V3`.
- Do not reopen parser / package / resolver / typecheck milestone scope unless a
  real new bug appears.
