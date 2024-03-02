# Ownership

Much like [C++]() and [Rust](), in Fol every variable declared, by default is created in stack unless explicitly specified othervise. Using option `[new]` or `[@]` in a variable, it allocates memory in the heap. The size of the allocation is defined by the type. Internally this creates a pointer to heap address, and dereferences it to the type you are having. Usually those behind the scene pointers here are unique pointers. This means that when the scope ends, the memory that the pointer used is freed.
```
var[new] intOnHeap: int[64];
@var intOnHeap: int[64];
```
## Assignments

[As discussed before](/docs/spec/040_variables/#assignments), declaring a new variable is like this:
```
var[pub] aVar: int[32] = 64
```
{{% notice warn %}}

However, when new variable is created and uses an old variable as value, the value is always cloned for "stack" declared values, but moved for "heap" declared values.

{{% /notice %}}

```
@var aVar: int[32] = 64
{
    var bVar = aVar                                              // this moves the content from $aVar to $bVar
}
.echo(aVar)                                                      // this will throw n error, since the $aVar is not anymore owner of any value
```

When the variable is moved, the owner is changed. In example above, the value `64` (saved in stack) is owned my `aVar` and then the ownership is moved to `bVar`. Now `bVar` is the new owner of the variable, making the `aVar` useless and can't be refered anymore. Since the `bVar` now controls the value, it's lifetime lasts until the end of the scope. When the scope ends, the variable is destroyed with `.de_alloc()` function. This because when the ovnership is moved, the attributes are moved too, so the `@` of `aVar` is now part of the `bVar` even if not implicitly specified. To avoid destruction, the `bVar` needs to return the ownership back to `aVar` before the scope ends with `.give_back(bVar)` or `!bVar`.

```
@var aVar: int[32] = 64
{
    var bVar = aVar                                              // this moves the content from $aVar to $bVar
    !bvar                                                        // return ownership
}
.echo(aVar)                                                      // this now will print 64
```
This can be done automatically by using [borrowing](/docs/spec/040_variables//#borrowing). 

## Borrowing
Borrowing does as the name says, it borrows a value from another variable, and at the end of the scope it automatically returns to the owner.

```
pro[] main: int = {
    var[~] aVar: int = 55;
    {
        var[bor] newVar: int = aVar                              // represents borrowing
        .echo(newVar)                                            // this return 55
    }
    .echo(aVar)                                                  // here $aVar it not accesible, as the ownership returns at the end of the scope
    .echo(newVar)                                                // we cant access the variable because the scope has ended
}
```
Borrowing uses a predefined option `[bor]`, which is not conventional like other languages that use `&` or `*`. This because you can get away just with "borrowing" without using pointers (so, symbols like `*` and `&` are strictly related to pointers)

However, while the value is being borrowed, we can't use the old variable while is being borrowed but we still can lend to another variable:
```
pro[] main: int = {
    var[~] aVar: int = 55;
    {
        var[bor] newVar = aVar                                   // represents borrowing
        .echo(newVar)                                            // this prints 55
        .echo(aVar)                                              // this throws an error, cos we already have borrowd the value from $aVar
        var[bor] anotherVar = aVar                               // $anotherVar again borrows from a $aVar
    }
}
```
{{% notice warn %}}

When borrowed, a the value is read-only (it's immutable). To make it muttable, <em>firtsly</em>, the owner needs to be muttable, <em>secondly</em> the borrower needs to declarare that it intends to change. 

{{% /notice %}} 

To do so, the borrower uses `var[mut, bor]`. However, when the value is declared mutable by owner, only one borrower within one scope can declare to modify it:
```
pro[] main: int = {
    var[~] aVar: int = 55;
    {
        var[mut, bor] newVar = aVar                              // [mut, bor] represents a mutable borrowing
        var[mut, bor] anotherVar = aVar                          // this throws an error, cos we already have borrowed the muttable value before
    }
    {
        var[mut, bor] anotherVar = aVar                          // this is okay, s it is in another scope
    }
}
```
