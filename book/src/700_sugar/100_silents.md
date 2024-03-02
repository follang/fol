# Silents


**Si**ngle **le**tter ide**nt**ifiers (SILENTs) identifiers are a form of languages sugar assignment.

## Letter

### Lowercase

Many times is needed to use a variable in-place and to decluter the code we use *silents*:

```
each(var x: str; x in {..10}){
    // implementation
}

each(x in {..10}){                      // we use the sicale `x` here
    // implementation
}
```
### Uppercase

If a *silent* is uppercase, then it is a constant, can't be changed. This is very important when using FOL for logic programming:

```
log vertical(l: line): bol = {
    l[A:B] and                          // we assign sicales `A` and `B`
    A[X:Y] and                          // we assign sicales `X` and `Y`
    B[X:Y2]                             // here we assign only `Y2` becase `X` exists from before
}
```

## Symbols

### Meh

**Meh** is the `_` identifier. The use of the term "meh" shows that the user is apathetic, uninterested, or indifferent to the question or subject at hand. It is occasionally used as an adjective, meaning something is mediocre or unremarkable.

We use **meh** when we want to discard the variable, or we dont intend to use:

```
var array: arr[int, 3] = {1, 2, 3};
var a, _, b: int = array;               // we discard, the middle value
```
### Y'all

**Y'all** is the `*` identifier. It represents app possible values that can be.
```
when(true) {
    case (x == 6){ // implementation }
    case (y.set()){ // implementation } 
    * { // default implementation }
}
```
