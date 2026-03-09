# Type System

This section defines the built-in type families used throughout the language.

Every expression has a type, and every declaration that introduces a value or callable surface interacts with the type system.

The built-in type families are grouped as:
- ordinal types:
  integers, floats, booleans, characters
- container types:
  arrays, vectors, sequences, matrices, maps, sets
- complex types:
  strings, numeric abstractions, pointers, errors
- special types:
  optional, union-like/sum-style surfaces, any-like and none-like forms

User-defined type construction is described later in the declarations section under `typ`, `ali`, records, entries, and standards.
