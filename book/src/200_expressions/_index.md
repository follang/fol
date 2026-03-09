# Statements And Expressions

This section covers executable syntax.

FOL separates executable forms into two broad groups:
- statements:
  forms executed for control flow, side effects, or declaration within a block
- expressions:
  forms that compute a value

## Statements

Statements include:
- local declarations
- assignments
- routine calls used for side effects
- branching
- looping
- nested blocks

Examples:

```fol
var x: int = 0;
x = 5;
if (x > 0) { .echo(x) }
for (item in items) { .echo(item) }
```

## Expressions

Expressions include:
- operator expressions
- literal expressions
- range expressions
- access expressions
- call expressions

Expressions can be nested freely and may appear in declarations, assignments, return statements, control-flow headers, and other expressions.
