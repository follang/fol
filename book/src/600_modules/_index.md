# Modules And Source Layout

This section defines how FOL source is organized across files, folders, packages, imports, and named module-like declarations.

It covers:
- imports through `use`
- namespaces and package layout
- block-like named definitions
- test-oriented module surfaces

At a high level:
- files in the same package contribute to the same package surface
- namespaces are expressed through folder structure and `::` access
- imported sources are classified by source kind such as `loc`, `pkg`, and `std`
