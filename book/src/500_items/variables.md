---
title: 'Variables'
type: "docs"
weight: 40
---

Here are some of the ways that variables can be defined:
```
var[pub,mut] somename: num[i32] = 98;
var[pub,exp] snotherone: str = "this is a string"
var[~] yetanother = 192.56
var[+] shortlet = true
var anarray: arr[str,3] = { "one", "two", "three" }
var asequence : seq[num[i8]] = { 20, 25, 45, 68, 73,98 }
var multiholder: set[num, str] = { 12, "word" }
var anothermulti: set[str, seq[num[f32]]] = { "string", {5.5, 4.3, 7, .5, 3.2} }
var shortvar = anothermulti[1][3]
var anymulti: any = {5, 10, "string", {'a',"word",{{0, "val"},{1, "nal"}}}, false}
var getSomeVal = anymulti[3][2][0][0] | < 15 | shortvar
```

## Assignments

Following the general rule of **FOL**:
```
declaration[options] name: type[options] = { implementation; };
```
then declaring a new variable is like this:
```
var[pub] aVar: int[32] = 64
```

however, the short version can be used too, and the compiler figures out at compute time the type:
```
var shortVar = 24;                      // compiler gives this value of `int[arch]`
```

When new variable is created, and uses an old variable to assign, the value is cloned, not referenced:	
```
pro[] main: int = {
    var aVar: int = 55;
    var newVar: int = aVar;
    .assert(&aVar == &newVar)           // this will return false
}
```
Two variables can not have the same memory location, unless we either borrow, or use pointers.

Variables can be assigned to an output of a function:
```
pro[] main: int = {
    fun addFunc(x, y: int): int = {
        return x + y;
    }
    var aVar: int = addFunc(4, 5);
}
```
### Piping / Ternary

Piping can be used as ternary operator. More about piping can be [found here](/docs/spec/pipes). Here is an example, the code below basically says: **if the function internally had an error, don't exit the program, but assign another value (or default value) to the variable**:
```
pro[] main: int = {
    fun addFunc(x, y: int): int = {
        return x + y;
    }
    var aVar: int = addFunc(4, 5) | result > 8 | return 6;
}
```
### Borrowing

If we want to reference a variable, the easiest way is to borrow the variable, use inside another scope (or the same) and return it back. If the ownership is not returned manually, by the end of the scope, it gets returned automatically. 
```
pro[] main: int = {
    var[~] aVar: int = 55;
    {
        var[bor] newVar: int = aVar         // var[bor] represents borrowing
        .echo(newVar)                       // this return 55
    }
        .echo(aVar)                         // here $aVar it not accesible, as the ownership returns at the end of the scope
        .echo(newVar)                       // we cant access the variable because the scope has ended
}
```
More on borrowing you can find [here](/docs/spec/pointers/#borrowing)

## Options
As with all other blocks, `var` have their options: `var[opt]`:

Options can be of two types: 
  - flags eg. `var[mut]` 
  - values eg. `var[pri=2]`

Flag options can have symbol aliases eg. `var[mut]` is the somename as `var[~]`.

```
|  opt   | s |   type    | description                                       | control       |
----------------------------------------------------------------------------------------------
|  mut   | ~ |   flag    | making a variable mutable                         | mutability    |
|  imu   |   |   flag    | making a variable imutable (default)              |               |
|  sta   | ! |   flag    | making a variable a static                        |               |
|  rac   | ? |   flag    | making a variable reactive                        |               |
----------------------------------------------------------------------------------------------
|  exp   | + |   flag    | making a global variable pubic                    | visibility    |
|  nor   |   |   flag    | making a global variable normal (default)         |               |
|  hid   | - |   flag    | making a global variable hidden                   |               |
```

### Alternatives
There is a shorter way for variables using alternatives, for example, instead of using `var[+]`, a leaner `+var` can be used instead.
```
def shko: mod[] = {
    +var aVar: int = 55;
    pro[] main: int { .echo(aVar) }
}
```
However, when we use two option in varable, only one can use the alternative form, so instead of using `var[mut,exp]`, this can be used `+var[mut]` or `+var[~]`, or vice varsa `~var[exp]` or `~var[+]`:
```
def shko: mod[] = {
    +var[mut] aVar: int = 55;
    pro[] main: int { .echo(aVar) }
}
```
## Types

### Immutable types (constants)
By default when a variable is defined without options, it is immutable type, for example here an intiger variable:
```
pro[] main: int = {
    var aNumber: int = 5;
    aNumber = 54;                       // reassigning varibale $aNumber thorws an error
}
```
### Mutable types
If we want a variable to be mutable, we have to explicitly pass as an option to the variable `var[mut]` or `var[~]`:
```
pro[] main: int = {
    var[mut] aNumber: int = 5
    var[~] anotherNumber: int = 24
    aNumber, anotherNumber = 6          // this is completely fine, we assign two wariables new values
}
```
### Reactive types
Reactive types is a types that flows and propagates changes. 

For example, in an normal variable setting, `var a = b + c` would mean that `a` is being assigned the result of `b + c` in the instant the expression is evaluated, and later, the values of `b` and `c` can be changed with no effect on the value of `a`. On the other hand, declared as reactive, the value of `a` is automatically updated whenever the values of `b` or `c` change, without the program having to re-execute the statement `a = b + c` to determine the presently assigned value of `a`.
```
pro[] main: int = {
    var[mut] b, c = 5, 4;
    var[rac] a: int = b + c
    .echo(a)                            // prints 9
    c = 10;
    .echo(a)                            // now it prints 10
}
```
### Static types
Is a variable which allows a value to be retained from one call of the function to another, meaning that its lifetime declaration. and can be used as `var[sta]` or `var[!]`. This variable is special, because if it is initialized, it is placed in the [data segment](https://en.wikipedia.org/wiki/Data_segment) (aka: initialized data) of the program memory. If the variable is not set, it is places in [.bss segmant](https://en.wikipedia.org/wiki/.bss) (aka: uninitialized data)
```
pro[] main: int = {
    {
        var[!] aNumber: int = 5
    }
    {
        .echo(aNumber)                  // it works as it is a static variable.
    }
}
```

### Scope

As disscussed before, files with same name share the same functions and global variables. However, those variables and functions can't be accesed if the whole module is imported in another project. In order for a variable to be accest by the importer, it needs to be have the `exp` flag option, so `var[exp]`, or `var[+]`

*module **shko**, file1.fol*
```
def shko: mod[] = {
    fun[+] add(a, b: int) = { return a + b }
    fun sub(a, b: int) = { return a - b }
}
```
*module **vij**, file1.fol*
```
use shko: mod[loc] = {../folder/shko}

def vij: mod[] = {
    pro[] main: int { 
        .echo(add( 5, 4 ))              // this works perfectly fine, we use a public/exported function
        .echo(sub( 5, 4 ))              // this throws an error, we are trying use a function that is not visible to other libraries
    }
}
```
There is even the opposite option too. If we want a function/variable to be only used inside the file ( so same package but only for that file ) then we use `hid` option flag: `var[hid]` or `var[-]`

*file1.fol*
```
def shko: mod[] = {
    var[-] aVar: str = "yo, sup!"
}
```

*file2.fol*
```
def shko: mod[] = {
    pro[] main: int { .echo(aVar) }       // this will thro an error (cos $aVar is declared private/hidden)
}
```
## Multiple

### Many to many

Many variables can be assigned at once, This is especially usefull, if variables have same options but different types eg. variable is mutabe and exported:

```
~var[exp] oneVar: int[32] = 24, twoVar = 13, threeVar: string = "shko";
```

Or to assign multiple variables of the same type:
```
~var[exp] oneVar, twoVar: int[32] = 24, 13;
```

To assign multiple variables of multiple types, the type is omitted, however, this way we can not put options on the type (obviously, the default type is assign by compiler):
```
~var[exp] oneVar, twoVar, threeVar = 24, 13, "shko";
```

Another "shameless plagiarism" from golang can be used by using `( ... )` to group variables:
```
~var[exp] (
    oneVar: int[32] = 13,
    twoVar: int[8] = 13,
    threeVar: str = "shko",
)
```

### Many to one

Many variables of the same type can be assigned to one output too:

```
var oneVar, twoVar: int[8] = 2;
```
However, each of them gets a copy of the variable on a new memory address:

```
.assert(&oneVar == &twoVar)           // this will return false
```

### One to many

And lastly, one variable can be assigned to multiple ones. This by using container types:
```
oneVar grouppy: seq[int] = { 5, 2, 4, 6 }
```
Or a more complicated one:
```
var anothermulti: set[str, seq[num[f32]]] = { "string", {5.5, 4.3, 7, .5, 3.2} }
```

Or a very simple one:
```
var simplemulti: any = { 5, 6, {"go", "go", "go"} }
```

## Containers

Containers are of special type, they hold other types within. As described before, there are few of them

### Access
To acces container variables, brackets like this `[]` are use:
```
var shortvar = anothermulti[1][3]     // compiler will copy the value `anothermulti[1][3]` (which is a float) to a new memory location
```

