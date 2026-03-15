# FOL Lowering Milestone Record

Last updated: 2026-03-15

This file is now a completion record for the `fol-lower` milestone.

## Scope That Was Completed

The completed lowering stage covers the current supported `V1` compiler subset:

- typed workspace input through `fol-typecheck`
- whole-workspace lowering across entry, `loc`, `std`, and installed `pkg` packages
- lowering-owned IDs, type tables, source maps, package metadata, and mounted ownership
- lowered globals, routine shells, locals, blocks, instructions, and terminators
- explicit lowering for:
- literals
- local/global loads
- assignments
- plain and qualified calls
- method calls
- field access
- index access
- `return`
- `report`
- statement and value `when`
- condition loops
- `break`
- record construction
- entry construction
- array/vector/sequence/set/map literals
- `nil`
- `unwrap`
- shell lifting and shell lowering surfaces already accepted in current `V1`
- workspace export metadata and entry routine candidates
- deterministic lowered snapshot output through `fol --dump-lowered`
- explicit lowering diagnostics for intentionally unsupported lowered surfaces
- verifier checks for basic CFG shape and dangling lowered references

## Validation Baseline

Latest full validation at milestone close:

- `make build`: passed
- `make test`: passed
- `8` unit tests passed
- `1508` integration tests passed

## Resulting Compiler Chain

The implemented pipeline at head is now:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower`

That means the repository now has:

- a hardened front end
- package-aware loading
- whole-workspace name resolution
- full current `V1` type checking
- a deterministic backend-facing lowered IR

## Explicit Boundaries

This completed milestone does not claim:

- a backend
- binary emission
- linking
- runtime packaging
- LLVM integration
- C backend generation
- ownership / borrowing
- standards / blueprints / generics
- C ABI

Those remain later milestones.

## Next Recommended Work

The next compiler step should be the first real backend that consumes lowered IR.

Reasonable next options:

- a simple C backend
- an LLVM backend
- another backend-neutral code-generation path

That next plan should start from the lowered IR contract now implemented here,
not from parser or typechecker structures directly.
