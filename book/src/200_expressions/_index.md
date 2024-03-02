---
title: "Code blocks"
description: 
draft: false
collapsible: true
weight: 200
---


## Statements

The actions that a program takes are expressed in statements. Common actions include declaring variables, assigning values, calling routine, looping through collections, and branching to one or another block of code, depending on a given condition. The order in which statements are executed in a program is called the flow of control or flow of execution. The flow of control may vary every time that a program is run, depending on how the program reacts to input that it receives at run time.

A statement can consist of a single line of code that ends in a semicolon, or a series of single-line statements in a block. A statement block is enclosed in {} brackets and can contain nested blocks. A statement in programming language theory usually results in something called a side effect. A side effect, loosely defined, is a permanent change of state in a program, such as modifying a global variable or changing the buffer stack.

Types of statement:
- declaration
- control


Here are some statements:
```
var x: int;                            // Also a declaration.
x = 0;                                 // Also an assignment.
if(expr) { /*...*/ }                   // This is why it's called an "if-statement".
for(expr) { /*...*/ }                  // For-loop.
```

## Expressions

An expression is a sequence of operators and their operands, that specifies a computation. It is a sequence of one or more operands and zero or more operators that can be evaluated to a single value, object, routine, or namespace. Expressions can consist of a literal value, a routine invocation, an operator and its operands, or a simple name. Simple names can be the name of a variable, type member, routine parameter, namespace or type.

Expressions can use operators that in turn use other expressions as parameters, or routine calls whose parameters are in turn other routine calls, so expressions can range from simple to very complex.

Types of expressions are divided into two groups:
- calculations
- literals
- ranges
- access
