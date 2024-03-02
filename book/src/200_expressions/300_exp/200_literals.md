# Literals

A literal expression consists of one or more of the numerical/letter forms described earlier. It directly describes a numbers, characters, booleans, containers and constructs.

There are two type of literals:
- values
- calls


## Value literals

Value literals are the simpliest expressions. They are direct values assigned to variables and are divided into two types:
- singletons
- clusters

### Singelton literals

Singleton literals represent one sigle values:

```
4                       // intiger literal
0xA8                    // hex-intiger literal
4.6                     // floating-point literal
5i                      // imaginary literal
"c"                     // character literal
"one"                   // string literal
true                    // boolean literal
```

### Cluster literals

Cluster literals represent both container types and construct types. Cluster literals are always enclosed within curly brackets `{  }`. The difference between scopes and cluster literals is that cluster literals shoud always have  comma `,` within the initializaion and assignment brackets, e.g `{ 5, }`.

#### Containers
Some simple container expressions
```
{ 5, 6, 7, 8, }                     // array, vector, sequences
{ "one":1, "two":2, }               // maps
{ 6, }                              // single element container
```
A 3x3x3 matrix
```
{{{1,2,3},{4,5,6},{7,8,9}},{{1,2,3},{4,5,6},{7,8,9}},{{1,2,3},{4,5,6},{7,8,9}}}

```

#### Constructs
```
// constructs 
{ email = "someone@example.com", username = "someusername123", active = true, sign_in_count = 1 }

// nested constructs
{
    FirstName = "Mark",
    LastName =  "Jones",
    Email =     "mark@gmail.com",
    Age =       25,
    MonthlySalary = {
        Basic = 15000.00,
        Bonus = {
            HTA =    2100.00,
            RA =   5000.00,
        },
    },
}
```
## Call literals

Call literals are function calls that resolve to values:
```
var seven: int = add(2, 5);             // assigning variables "seven" to function call "add"
```


`typ Vector: rec = {
    var x: flt
    var y: flt
}

typ Rect: rec = {
    var pos: Vector
    var size: Vecotr
}


fun make_rect(min, max: Vector): Rect {
    return [Rect]{{min.x, min.y}, {max.x - max.y, max.y - max.y}}
    return [Rect]{pos = {min.x, min.y}, size = {max.x - max.y, max.y - max.y}}
}



`
