---
title: 'Functions'
type: "docs"
weight: 20
---

Functions compared to procedure are pure. A pure function is a function that has the following properties:

- Its return value is the same for the same arguments (no variation with local static variables, non-local variables, mutable reference arguments or input streams from I/O devices).
- Its evaluation has no side effects (no mutation of local static variables, non-local variables, mutable reference arguments or I/O streams).

Thus a pure function is a computational analogue of a mathematical function. Pure functions are declared with `fun[]`
```
fun[] add(el1, el2: int[64]): int[64] = { result = el1 + el2 }
```
{{% notice warn %}}

Functions in FOL are lazy-initialized. 

{{% /notice %}}

So it is an evaluation strategy which delays the evaluation of the function until its value is needed. You call a function passing it some arguments that were expensive to calculate and then the function don’t need all of them due to some other arguments.

Consider a function that logs a message:
```
log.debug("Called foo() passing it " + .to_string(argument_a) + " and " + .to_string(argument_b));
```
The log library has various log levels like “debug”, “warning”, “error” etc. This allows you to control how much is actually logged; the above message will only be visible if the log level is set to the “debug” level. However, even when it is not shown the string will still be constructed and then discarded, which is wasteful.


{{% notice tip %}}

Since Fol supports first class functions, it allows functions to be assigned to variables, passed as arguments to other functions and returned from other functions.

{{% /notice %}}

### Anonymous functoins

Anonymous function is a function definition that is not bound to an identifier. These are a form of nested function, in allowing access to variables in the scope of the containing function (non-local functions).

Staring by assigning a anonymous function to a vriable:
```
var f = fun (a, b: int): int = {                                        // assigning a variable to function
    return a + b
}
.echo(f(5,6))                                                           // prints 11

var f: int = (a, b: int){                                               // this is an short alternative of same variable assignmet to function
    return a + b
}
```

It is also possible to call a anonymous function without assigning it to a variable.
```
`version 1`
fun[] (a, b: int) = {                                                   `define anonymous function`
    .echo(a + b)
}(5, 6)                                                                 `calling anonymous function`


`version 2`
(a, b: int){                                                            `define anonymous function`
    .echo(a + b)
}(5, 6)                                                                 `calling anonymous function`
```


### Closures
Functions can appear at the top level in a module as well as inside other scopes, in which case they are called nested functions. A nested function can access local variables from its enclosing scope and if it does so it becomes a closure. Any captured variables are stored in a hidden additional argument to the closure (its environment) and they are accessed by reference by both the closure and its enclosing scope (i.e. any modifications made to them are visible in both places). The closure environment may be allocated on the heap or on the stack if the compiler determines that this would be safe. 

There are two types of closures:
- anonymous
- named

Anonymus closures automatically capture variables, while named closures need to be specified what to capture. For capture we use the `[]` just before the type declaration.
```
fun[] add(n: int): int = {
    fun added(x: int)[n]: int = {                                       // we make a named closure 
        return x + n                                                    // variable $n can be accesed because we have captured ti
    }    
    return adder()
}

var added = add(1)                                                      // assigning closure to variable
added(5)                                                                // this returns 6
```

```
fun[] add(n: int): int = {
    return fun(x: int): int = {                                         // we make a anonymous closure 
        return x + n                                                    // variable $n can be accesed from within the nested function
    }
}
```

### Currying
Currying is converting a single function of "n" arguments into "n" functions with a "single" argument each. Given the following function:
```
fun f(x,y,z) = { z(x(y));}
```
When curried, becomes:
```
fun f(x) = { fun(y) = { fun(z) = { z(x(y)); } } }
```
 And calling it woud be like:
 ```
f(x)(y)(z)
 ```
However, the more iportant thing is taht, currying is a way of constructing functions that allows partial application of a function’s arguments. What this means is that you can pass all of the arguments a function is expecting and get the result, or pass a subset of those arguments and get a function back that’s waiting for the rest of the arguments. 
 ```
fun calc(x): int = {
    return fun(y): int = {
        return fun (z): int = {
            return x + y + z
        } 
    }
}

var value: int = calc(5)(6)                                             // this is okay, the function is still finished
var another int = value(8)                                              // this completes the function

var allIn: int = calc(5)(6)(8)                                          // or this as alternative
 ```

### Higer-order functions
A higher-order function is a function that takes a function as an argument. This is commonly used to customize the behavior of a generically defined function, often a looping construct or recursion scheme.

They are functions which do at least one of the following:

- takes one or more functions as arguments
- returns a function as its result

```
//function as parameter
fun[] add1({fun adder(x: int): int}): int = {
    return adder(x + n)
}

//function as return
fun[] add2(): {fun (x: int): int} = {
    var f = fun (a, b: int): int = {
        return a + b
    }    
    return f
}
```
### Generators
A generator is very similar to a function that returns an array, in that a generator has parameters, can be called, and generates a sequence of values. However, instead of building an array containing all the values and returning them all at once, a generator yields the values one at a time, which requires less memory and allows the caller to get started processing the first few values immediately. In short, a generator looks like a function but behaves like an iterator.

For a function to be a generator (thus to make the keyword `yeild` accesable), it needs to return a type of container: `arr, vec, seq, mat` but not `set, any`.
```
fun someIter: vec[int] = {
    var curInt = 0;
    loop(){
        yeild curInt.inc(1)
    }
}
```
