# Symbols

## Operators

Fol allows user defined operators. An operator is any combination of the following characters:
```
=     +     -     *     /     >     .
@     $     ~     &     %     <     :
!     ?     ^     #     `     \     _
```

The grammar uses the terminal `OP` to refer to operator symbols as defined here.



## Brackets

Bracket punctuation is used in various parts of the grammar. An open bracket must always be paired with a close bracket. Here are type of brackets used in FOL:

bracket | type | purpose
--- | --- | ---
`{  }` | Curly brackets | Code blocks, Namespaces, Containers
`[  ]` | Square brackets | Type options, Container acces, Multithreading
`(  )` | Round brackets | Calculations, Comparisons, Argument passing
`<  >` | Angle brackets


The grammar uses the terminal `BR` to refer to operator symbols as defined here.
