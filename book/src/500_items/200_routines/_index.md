# Routines

Routines are callable declarations.

FOL has three routine families:
- `pro`:
  procedures, used for effectful work
- `fun`:
  functions, intended for ordinary value-producing computation
- `log`:
  logical routines and relation-like callable forms

Routine declarations support a shared structural pattern:

```fol
fun[options] name(params): return_type = { body }
pro[options] name(params): return_type = { body }
log[options] name(params): return_type = { body }
```

FOL also allows an alternate header style:

```fol
fun[options] name: return_type = (params) { body }
```

This chapter family covers parameters, calls, defaults, variadics, return values, and routine-specific semantics.

## Types
There are two main types of routines in fol:

- Procedurues

    A procedure is a piece of code that is called by name. It can be passed data to operate on (i.e. the parameters) and can optionally return data (the return value). All data that is passed to a procedure is explicitly passed.

- Functions

    A function is called pure function if it always returns the same result for same argument values and it has no side effects like modifying an argument (or global variable) or outputting to I/O. The only result of calling a pure function is the return value.

## Parameters
### Formal parameters

Routines typically describe computations. There are two ways that a routine can gain access to the data that it is to process: through direct access to nonlocal variables (declared elsewhere but visible in the routine) or through parameter passing. Data passed through parameters are accessed using names that are local to the routine. Routine create their own unnamed namespace. Every routine has its own Workspace. This means that every variable inside the routine is only usable during the execution of the routine (and then the variables go away). 

Parameter passing is more flexible than direct access to nonlocal variables. Prrameters are special variables that are part of a routine’s signature. When a routine has parameters, you can provide it with concrete values for those parameters. The parameters in the routine header are called formal parameters. They are sometimes thought of as dummy variables because they are not variables in the usual sense: In most cases, they are bound to storage only when the routine is called, and that binding is often through some other program variables.

Parameters are declared as a list of identifiers separated by semicolon (or by a colon, but for code cleanness, the semicolon is preferred). A parameter is given a type by : typename. If after the parameter the `:` is not declared, but `,` colon to identfy another paremeter, of which both parameters are of the same type if after the second one the `:` and the type is placed. Then the same type parameters continue to grow with `,` until `:` is reached.
```
fun[] calc(el1, el2, el3: int[64]; changed: bol = true): int[64] = { result = el1 + el2 - el3 }
```

In routine signatures, you must declare the type of each parameter. Requiring type annotations in routine definitions is obligatory, which means the compiler almost never needs you to use them elsewhere in the code to figure out what you mean. Routine can parameter overloaded too. It makes possible to create multiple routine of the same name with different implementations. Calls to an overloaded routine will run a specific implementation of that routine appropriate to the context of the call, allowing one routine call to perform different tasks depending on context:

```
fun retBigger(el2, el2: int): int = { return el1 | this > el2 | el2 }
fun retBigger(el2, el2: flt): flt = { return el1 | this > el2 | el2 }

pro main: int = {
    retBigger(4, 5);                                        // calling a routine with intigers
    retBigger(4.5, .3);                                     // calling another routine with same name but floats
}
```
The overloading resolution algorithm determines which routine is the best match for the arguments. Example:
```
pro toLower(c: char): char = {                              // toLower for characters
    if (c in {'A' ... 'Z'}){
        result = chr(ord(c) + (ord('a') - ord('A')))
    } else {
        result = c
    }
}

pro toLower(s: str): str = {                                // toLower for strings
    result = newString(.len(s))
    for i in {0 ... len(s) - 1}:
        result[i] = toLower(s[i])                           // calls toLower for characters; no recursion!
}
```

### Actual parameters
routine call statements must include the name of the routine and a list of parameters to be bound to the formal parameters of the routine. These parameters are called actual parameters. They must be distinguished from formal parameters, because the two usually have different restrictions on their forms.

#### Positional parameters

The correspondence between actual and formal parameters, or the binding of actual parameters to formal parameters - is done by position: The first actual parameter is bound to the first formal parameter and so forth. Such parameters are called positional parameters. This is an effective and safe method of relating actual parameters to their corresponding formal parameters, as long as the parameter lists are relatively short. 

```
fun[] calc(el1, el2, el3: int): int = { result = el1 + el2 - el3 }

pro main: int = {
    calc(3,4,5);                                            // calling routine with positional arguments
}
```

#### Keyword parameters

When parameter lists are long, however, it is easy to make mistakes in the order of actual parameters in the list. One solution to this problem is with keyword parameters, in which the name of the formal parameter to which an actual parameter is to be bound is specified with the actual parameter in a call. The advantage of keyword parameters is that they can appear in any order in the actual parameter list. 

```
fun[] calc(el1, el2, el3: int): int = { result = el1 + el2 - el3 }

pro main: int = {
    calc(el3 = 5, el2 = 4, el1 = 3);                        // calling routine with keywords arguments
}
```
#### Mixed parameters
Keyword and positional arguments can be used at the same time too. The only restriction with this approach is that after a keyword parameter appears in the list, all remaining parameters must be keyworded. This restriction is necessary because a position may no longer be well defined after a keyword parameter has appeared.
```
fun[] calc(el1, el2, el3: int, el4, el5: flt): int = { result[0] = ((el1 + el2) * el4 ) - (el3 ** el5);  }

pro main: int = {
    calc(3, 4, el5 = 2, el4 = 5, el3 = 6);                  // element $el3 needs to be keyeorded at the end because 
                                                            // its positional place is taken by keyword argument $el5
}
```

### Default arguments
Formal parameters can have default values too. A default value is used if no actual parameter is passed to the formal parameter. The default parameter is assigned directly after the formal parameter declaration. The compiler converts the list of arguments to an array implicitly. The number of parameters needs to be known at compile time. 
```
fun[] calc(el1, el2, el3: rise: bool = true): int = { result[0] = el1 + el2 * el3 | this | el1 + el2;  }

pro main: int = {
    calc(3,3,2);                                            // this returns 6, last positional parameter is not passed but 
                                                            // the default `true` is used from the routine declaration
    calc(3,3,2,false)                                       // this returns 12
}
```

### Variadic routine
The use of `...` as the type of argument at the end of the argument list declares the routine as variadic. This must appear as the last argument of the routine. When variadic routine is used, the default arguments can not be used at the same time.
```
fun[] calc(rise: bool; ints: ... int): int = { result[0] = ints[0] + ints[1] + ints[2] * ints[3] | this | ints[0] + ints[1];  }

pro main: int = {
    calc(true,3,3,3,2);                                     // this returns 81, four parmeters are passed as variadic arguments
    calc(true,3,3,2)                                        // this returns 0, as the routine multiplies with the forth varadic parameter
                                                            // and we have given only three (thus the forth is initialized as zero)
}
```

`...` is called unpack operator - just like in Golang. In the routine above, you see `...`, which means pack all incoming arguments into `seq[int]` after the first argument. The sequence then is turned into a list at compile time.

{{% notice warn %}}

Nested procedures don't have access to the outer scope, while nested function have but can't change the state of it.

{{% /notice %}}

## Return

The return type of the routine has to always be defined, just after the formal parameter definition. Following the general rule of **FOL**: 
```
fun[] add(el1, el2: int[64]): int[64] = { result = el1 + el2 }
```

To make it shorter (so we don't have to type `int[64]` two times), we can use a *short form* by omitting the return type. The compiler then will assign the returntype the same as the functions return value.
```
fun[] add(el1, el2: int[64]) = { result = el1 + el2 }
```
{{% notice info %}}

Current `V1` routine summary:

- routines declare a success type after `:`
- routines may also declare a recoverable error type after `/`
- `report expr` exits through that declared error path
- routine call results declared with `/ ErrorType` are not `err[...]` shell
  values
- use propagation, `check(...)`, or `expr || fallback` for those calls
- keep postfix `!` for `opt[...]` and `err[...]` shell values

{{% /notice %}}

The implicitly declared variable `result` is of the same type of the return type. For it top be implicitly declared, the return type of the function shoud be always declared, and not use the short form. The variable is initialized with zero value, and if not changed during the body implementation, the same value will return (so zero).
```
pro main(): int = {
    fun[] add(el1, el2: int[64]): int[64] = { result = el1 + el2 }          // using the implicitly declared $result variable
    fun[] sub(el1, el2: int[64]) = { return el1 - el2 }                     // can't access the result variable, thus we use return
}
```
Recoverable error-aware routines use the current signature form:
```
fun[] read(path: str): int / str = {
    report "missing path"
}
```
and are handled at the call site with propagation, `check(...)`, or `||`
rather than shell unwrap.

Current intrinsic note:

- `.echo(...)` is a dot-root diagnostic intrinsic
- `check(...)` is a keyword intrinsic for recoverable-call inspection
- `panic(...)` is a keyword intrinsic for immediate abort
- `as` and `cast` are registry-owned but still deferred in current `V1`

The final expression in the function will be used as return value. For this to be used, the return type of the function needs to be defined (so the function cnat be in the short form)). ver this can be used only in one statement body.
```
pro main(): int = {
    fun[] add(el1, el2: int[64]): int[64] = { el1 + el2 }                   // This is tha last statement, this will serve as return
    fun[] someting(el1,el2: int): int = {
        if (condition) {

        } else {

        }
        el1 + el2                                                           // this will throw an error, cand be used in kulti statement body
    }
    fun[] add(el1, el2: int[64]) = { el1 + el2 }                            // this will throw an error, we can't use the short form of funciton in this way
```
Alternatively, `return` and `report` can exit a routine early from within
control flow. See the recoverable-error chapter for the full current `V1`
contract and the shell-vs-routine distinction.
