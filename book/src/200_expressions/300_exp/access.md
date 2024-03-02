---
title: "Access"
type: "docs"
weight: 500
---

There are four access expresions:
- namespace member access
- routine member access
- container memeber access
- field member access


## Subprogram access

In most programming languages, it is called "method-call expresion". A method call consists of an expression (the receiver) followed by a single dot `.`, an expression path segment, and a parenthesized expression-list:
```
"3.14".cast(float).pow(2);                  // casting a numbered string to float, then rising it to power of 2
```


## Namespaces access

Accesing namespaces is done through double colon operator `::`:
```
use log mod[std] = { fmt::log };            // using the log namespace of fmt
io::console::write_out.echo();              // echoing out
```


## Container access

### Array, Vectors, Sequences, Sets
Containers can be indexed by writing a square-bracket-enclosed expression of type `int[arch]` (the index) after them.
```
var collection: int = { 5, 4, 8, 3, 9, 0, 1, 2, 7, 6 }

collection[5]                               // get the 5th element staring from front (this case is 0)
collection[-2]                              // get the 3th element starting from back (this case is 1)
```

Containers can be accessed with a specified range too, by using colon within a square-bracket-enclosed:

syntax | meaning
--- | ---
`:` | the whole container
elA`:`elB | from element `elA` to element `elB`
`:`elA | from beginning to element `elA`
elA`:` | from element `elA` to end

```
collection[-0]                              // last item in the array
{ 6 }
collection[-1:]                             // last two items in the array
{ 7, 6 }
collection[:-2]                             // everything except the last two items
{ 5, 4, 8, 3, 9, 0, 1, 2 }
```

If we use double colon within a square-bracket-enclosed then the collection is inversed:

syntax | meaning
--- | ---
`::` | the whole container in reverse
elA`::`elB | from element `elA` to element `elB` in reverse
`::`elA | from beginning to element `elA` in reverse
elA`::` | from element `elA` to end in reverse

```
collection[::]                              // all items in the array, reversed
{ 6, 7, 2, 1, 0, 9, 3, 8, 4, 5 }
collection[2::]                             // the first two items, reversed
{ 4, 5 }
collection[-2::]                            // the last two items, reversed
{ 6, 7 }
collection[::-3]                            // everything except the last three items, reversed
{ 2, 1, 0, 9, 3, 8, 4, 5 }
```
### Matrixes
Matrixes are 2D+ arrays, thus they have a bit more complex acces way:
```
var aMat = mat[int, int] = { {1,2,3}, {4,5,6}, {7,8,9} };

nMat[[1][0]]                                // this will return 4
                                            // first [] accesses the first dimension, then second [] accesses the second
```
All other operations are the same like arrays.

### Maps
Accesing maps is donw by using the key within square-bracket-enclosed:
```
var someMap: map[str, int] = { {"prolog", 1}, {"lisp", 2}, {"c", 3} }
someMap["lisp"]                             // will return 2
```
### Axioms
Accesing axioms is more or less like accessing maps, but more verbose and matching through **backtracing**, and the return is always a vector of elements (empty if no elements are found):
```
var parent: axi[str, str] = { {"albert","bob"}, {"alice","bob"}, {"bob","carl"}, {"bob","tom"} };

parent["albert",*]                          // this will return strng vector: {"bob"}
parent["bob",*]                             // this will return strng vector: {"carl","tom"}

parent[*,_]                                 // this will match to {"albert", "alice", "bob"}
```

Matching can be with a vector too:
```
var parent: axi[str, str] = { {"albert","bob"}, {"alice","bob"}, {"bob","carl"}, {"bob","tom"}, {"maggie","bill"} };
var aVec: vec[str] = { "tom", "bob" };

parent[*,aVec]                              // will match all possible values that have "tom" ot "bob" as second element
                                            // in this case will be a strng vector: {"albert", "alice", "bob"}
```

a more complex matching:
```
var class: axi;
class.add({"cs340","spring",{"tue","thur"},{12,13},"john","coor_5"})
class.add({"cs340","winter",{"tue","fri"},{12,13},"mike","coor_5"})
class.add({"cs340",winter,{"wed","fri"},{15,16},"bruce","coor_3"})
class.add({"cs101",winter,{"mon","wed"},{10,12},"james","coor_1"})
class.add({"cs101",spring,{"tue","tue"},{16,18},"tom","coor_1"})

var aClass = "cs340"
class[aClass,_,[_,"fri"],_,*,_]             // this will return string vector: {"mike", bruce}
                                            // it matches everything that has aClass ad "fri" within
                                            // and ignore ones with meh symbol
```

### Avaliability
To check if an element exists, we add `:` before accessing with []. Thus this will return `true` if element exists.
```
var val: vec = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}

val:[5]                                     // returns true
val:[15]                                    // returns false


var likes: axi[str, str] = { {"bob","alice"} , {"alice","bob"}, {"dan","sally"} };

likes["bob","alice"]:                       // will return true
likes["sally","dan"]:                       // will return false
```

### In-Place assignment
One of the features that is very important in arrays is that they can assign variables immediately:
```
var val: vec = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
var even: vec = {2, 4, 6, 8, 10, 12, 14, 16, 18, 20}
val[even => Y]                              // this equals to {2, 4, 6, 8, 10} and same time assign to Y
.echo(Y)                                    // will print {2, 4, 6, 8, 10}
```
This dows not look very much interesting here, you can just as easy assign the whole filtered array to a variable, but it gets interesting for axioms:
```
var parent: axi[str, str] = { {"albert","bob"}, {"alice","bob"}, {"bob","carl"}, {"bob","tom"}, {"maggie","bill"} };

parent:[* => Y,"bob"]                       // this one returns true if we find a parent of "bob"
                                            // same time it assigns parent to a string vector `Y`
```


## Field access

Field access expressoin accesses fields inside constructs. Here is a recorcalled `user`:
```
var user1: user = {
    email = "someone@example.com",
    username = "someusername123",
    active = true,
    sign_in_count = 1
};

fun (user)getName(): str = { result = self.username; };
```
There are two types of fields that can be accesed within constructs:
- methods
- data

### Methods

Methods are accesed the same way like routine member access.
```
user1.getName()
```

### Data


There are multiple ways to acces data within the construct. The easiest one is by dot operator `.`:
```
user1.email                                 // accessing the field through dot-accesed-memeber
```

Another way is by using square bracket enclosed by name:
```
user1[email]                                // accessing the field through square-bracket-enclosed by name
```

And lastly, by square bracket enclosed by index:
```
user1[0]                                    // accessing the field through square-bracket-enclosed by index

```
