# Build-in

Fol has many build-in functions and macros offered by compiler, and you access them by . (with space/newline/bracket before):

```
var contPoint: ptr[int] = 10;			// make a pointer and asign the memory to value of 10
.print(.pointer_value(contPoint));		// print the dereferenced value of pointer
```

{{% placeholder %}}

`.echo()`               - print on screen 
`.not()`                - negate
`.cast()`               - type casting
`.as()`                 - type casting

`.eq()`                 - check for equality
`.nq()`                 - check for inequality
`.gt()`                 - greater than
`.lt()`                 - less than
`.ge()`                 - greater or equal
`.le()`                  - less or equal

`.de_alloc()`           - drop variable from scope
`.give_back()`          - return ownership

`.size_of()`            - variable type size
`.addres_of()`          - pointer address
`.pointer_value()`      - value of the pointer


{{% /placeholder %}}

