# Aliases

An alias declaration binds an identifier to an existing type. All the properties of the existing type are bound to the alias too.

There are two type of aliasing:
- aliasing
- extending

## Aliasing

```
typ[ali] I5: arr[int, 5];
```

So now the in the code, instead of writing `arr[int, 5]` we could use `I5`:

```
~var[exp] fiveIntigers: I5 = { 0, 1, 2, 3, 4, 5 }
```
Another example is creating a `rgb` type that can have numbers only form 0 to 255:
```
typ[ali] rgb: int[8][.range(255)] ;                        // we create a type that holds only number from 0 to 255
typ[ali] rgbSet: set[rgb, rgb, rgb];                       // then we create a type holding the `rgb` type
```

Alias declaration are created because they can simplify using them multiple times,
their identifier (their name) may be expressive in other contexts, and-most
importantly-so that you can define receiver-qualified routines on a named type
surface. Anonymous types still need a named alias or extension surface first,
while built-in or foreign types can be extended explicitly through `typ[ext]`.

Attaching methods does not make a type into an object. It simply gives a
routine a receiver-qualified dot-call spelling. You could still think of the
operation procedurally as a routine that takes the value as its first input.


## Extending

Extensions add new functionality to an existing constructs. This includes the ability to extend types for which you do not have access to the original source code (known as retroactive modeling).
```
typ[ext] type: type;
```
For example, adding a `print` function to the default integer type `int`:
```
typ[ext] int: int;

pro (int)print(): non = {
    .echo(self)
}

pro main: int = {
    5.print()                   // method print on int
}
```

Or turning a string `str` into a vector of characters:

```
typ[ext] str: str;

fun (str)to_array(): vec[chr] = {
    loop(x in self){
        yield x; 
    }
}


pro main(): int = {
    var characters: vec[chr] = "a random str".to_array();

    .echo(characters)           // will print: {"a"," ","r","a","n","d","o","m"," ","s","t","r"}
}

```
