This directory now contains one larger example package:

- `operations_console/`
- `full_v1_showcase/app/`

`operations_console` is a multi-file current-V1 example intended to exercise a realistic amount of surface area:

- sibling source files in one package
- subfolder namespaces
- records, entries, aliases, and methods
- containers and indexing
- intrinsics such as `.eq`, `.gt`, `.not`, `.len`, and `.echo`
- recoverable routines with `/ ErrorType`, `check(...)`, and `||` fallback

Compile it with:

```bash
./target/debug/fol examples/operations_console
```

`full_v1_showcase/app` is a broader current-V1 integration example built around a `loc` import:

- `loc` package import
- subfolder namespace access through `shared::models`
- records, entries, aliases, and methods
- recoverable routines, `check(...)`, `||`, `opt[...]`, `err[...]`, and `!`
- containers, indexing, and current intrinsic families

Compile it with:

```bash
./target/debug/fol examples/full_v1_showcase/app
```
