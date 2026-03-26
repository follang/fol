# Build Direction

This note records the near-term direction of the FOL build system.

It is not a promise to copy another build system exactly. It is the current
shape we intend to grow into from the existing `build.fol` surface.

## Current Shape

Today the public layering is:

- `.build()` for the package build context
- `build.meta({...})` for package metadata
- `build.add_dep({...})` for direct dependencies
- `build.graph()` for artifact and step graph mutation

This already covers the core FOL build contract:

- one control file
- one canonical `pro[] build(): non`
- package metadata and dependencies in the build language
- graph-based artifact construction

## What Is Still Missing

The next useful pieces are not more random graph methods. They are missing
capability classes.

### Dependency Handles

Direct dependencies should not stay only as metadata declarations.

The intended shape is:

```fol
var logtiny = build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
});
```

and then:

```fol
var logtiny_mod = logtiny.module("logtiny");
var logtiny_lib = logtiny.artifact("logtiny");
var logtiny_gen = logtiny.generated("bindings");
```

That makes dependencies real build values instead of only package-loader data.

The current dependency import model stays explicit:

- ordinary source imports still resolve by alias projection under `.fol/pkg/<alias>`
- dependency handles query the build-facing surface of that package

The next public layer on top of that is explicit exports from the dependency
package itself:

```fol
build.export_module({ name = "api", module = codec });
build.export_artifact({ name = "runtime", artifact = lib });
build.export_step({ name = "check", step = docs });
build.export_output({ name = "schema", output = bindings });
```

Projection remains the fallback during the transition, but explicit exports are
the preferred public contract.

This keeps source imports and build-surface queries separate instead of
collapsing them into one implicit registry.

### Output Handles

Generated and copied files should not remain fragmented into unrelated special
cases.

The intended capability is one output-handle family that can represent:

- files written by `graph.write_file(...)`
- files copied by `graph.copy_file(...)`
- captured stdout from run/system-tool/codegen steps
- generated files exported by dependency packages

This should behave like one composable build value, not a pile of unrelated
string paths. That handle family is now the current direction for local and
dependency-generated outputs alike.

### Explicit Dependency Arguments

Dependencies should eventually accept explicit forwarded build arguments:

```fol
var dep = build.add_dep({
    alias = "json",
    source = "pkg",
    target = "json",
    args = {
        target = graph.standard_target(),
        optimize = graph.standard_optimize(),
        use_fast_parser = true,
    },
});
```

The important rule is explicitness:

- no ambient forwarding
- no hidden inheritance
- the package author chooses what to pass

### Install Prefix

The build graph should describe what gets installed, but the output prefix
should remain a user/tool choice rather than something hardcoded into the
package.

That means FOL should continue to separate:

- build/cache internals
- fetched dependency storage
- final installed outputs

### Step Execution

Step execution is still serial today. The current work is about making cache
boundaries and reporting honest and explicit, not pretending the executor is
already parallel.

The current reporting direction is:

- requested
- executed
- skipped-from-cache
- skipped-by-foreign-run-policy
- produced outputs

## What This Does Not Mean

This direction does not mean:

- reintroducing public `Graph` or `Build` types
- collapsing package metadata back into YAML
- making dependency behavior implicit
- turning the graph API into a stringly catch-all
- introducing a separate build manifest file
- pretending step execution is parallel before it is

The current design constraint remains:

- package metadata through `build.meta`
- direct dependencies through `build.add_dep`
- graph mutation through `build.graph`

The next round should add richer values on top of that layering, not replace it.
