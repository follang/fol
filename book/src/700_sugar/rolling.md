---
title: 'Rolling'
type: "docs"
weight: 500
---

Rolling or list comprehension is a syntactic construct available FOL for creating a list based on existing lists. It follows the form of the mathematical set-builder notation - set comprehension.


Rolling has the same syntactic components to represent generation of a list in order from an input list or iterator:

- A variable representing members of an input list.
- An input list (or iterator).
- An optional predicate expression.
- And an output expression producing members of the output list from members of the input iterable that satisfy the predicate.

The order of generation of members of the output list is based on the order of items in the input. Syntactically, rolling consist of an iterable containing an expression followed by a for statement. In FOL the syntax follows exacly the **Python's list comprehension** syntax:
```
var aList: vec[] = { x for x in iterable if condition }
```

Rolling provides an alternative syntax to creating lists and other sequential data types. While other methods of iteration, such as for loops, can also be used to create lists, rolling may be preferred because they can limit the number of lines used in your program.
```
var aList: vec[] = {..12};

var another: vec[] = { ( x * x ) for ( x in aList ) if ( x % 3 == 0 ) }
var matrix: mat[int, int] = { x * y for ( x in {..5}, y in {..5} ) }
```


