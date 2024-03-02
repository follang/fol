---
title: "Type system"
description: 
draft: false
collapsible: true
weight: 400
---

A data type defines a collection of data values and a set of predefined operations on those values. Computer programs produce results by manipulating data. An important factor in determining the ease with which they can perform this task is how well the data types available in the language being used match the objects in the real world of the problem being addressed. Therefore, it is crucial that a FOL supports an appropriate collection of data types and structures.


The type system of a programming language defines how a type is associated with each expression in the language and includes its rules for type equivalence and type compatibility. Certainly, one of the most important parts of understanding the semantics of a programming language is understanding its type system.

Data types that are not defined in terms of other types are called primitive data types. Nearly all programming languages provide a set of primitive data types. Some of the primitive types are merely reflections of the hardware for example, most integer types. Others require only a little nonhardware support for their implementation.


## Types

Every value in Fol is of a certain data type, which tells Fol what kind of data is being specified so it knows how to work with that data. 

Types are divided into two groups:
- perdefined
- constructed


### Predefned types
There are four predefned types: 

- ordinal ( integer, float, boolean, character )
- container ( array, vector, sequence, matrix, map, set )
- complex  (string, number, pointer, error )
- special ( optional, never, any, null )

### Constructed types
- records
- tables
- entries

