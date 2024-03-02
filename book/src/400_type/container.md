---
title: 'Container'
type: "docs"
weight: 200
---

Containers are of compound types. They contain other primitive or constructed types. To access the types in container those brackets are used: `[]`, so:
```
var container: type = { element, element, element }             // declaring a container
var varable: type = container[2]                                // accessing the last element
```


{{% notice note %}}

Containers are always zero indexed

{{% /notice %}}


### Static Arrays
#### Arrays
```
arr[type,size]
```
Arrays are the most simple type of container. They contain homogeneous type, meaning that each element in the array has the same type. Arrays always have a fixed length specified as a constant expression `arr[type, size]`. They can be indexed by any ordinal type to acces its members.

```
pro[] main: int = {
    var anArray: arr[int, 5] = { 0, 1, 2, 3, 4 };             // declare an array of intigers of five elements
    var element = anArray[3];                                 // accessing the element
    .echo(element)                                            // prints: 3
}
```

To allocate memory on heap, the `var[new]` is used [ more about memory, ownreship and pointer ](/std/spec/050_pointers/):
```
pro[] main: int = {
    var[new] aSequence: arr[str] = { "get", "over", "it" };   // this array is stored in stack
}
```

### Dynamic arrays

Dynamic are similar to arrays but of dynamic length which may change during runtime (like strings). A dynamic array `s` is always indexed by integers from `0` to `.len(s)-1` and its bounds are checked.
The lower bound of an array or sequence may be received by the built-in `.low()`, the higher bound by `.high()`. The length may be received by `.len()`.

{{% notice tip %}}

Dynamic arrays are a dynamically allocated (hence the name), thus if not allocated in heap but in stack, the size will be defined automatically in compile time and will be changed to static array.

{{% /notice %}}


There are two implementations of dynamic arrays:
- vectors `vec[]`
- sequences `seq[]`



#### Vecotors

Vectors are dynamic arrays, that resizes itself up or down depending on the number of content.

Advantage:

- accessing and assignment by index is very fast O(1) process, since internally index access is just [address of first member] + [offset].
- appending object (inserting at the end of array) is relatively fast amortized O(1). Same performance characteristic as removing objects at the end of the array. Note: appending and removing objects near the end of array is also known as push and pop.

Disadvantage:

- inserting or removing objects in a random position in a dynamic array is very slow O(n/2), as it must shift (on average) half of the array every time. Especially poor is insertion and removal near the start of the array, as it must copy the whole array.
- Unpredictable performance when insertion or removal requires resizing
- There is a bit of unused space, since dynamic array implementation usually allocates more memory than necessary (since resize is a very slow operation)

In FOL vecotrs are represented like this:
```
vec[type]
```
Example:
```
pro[] main: int = {
    var[new] aSequence: seq[str] = { "get", "over", "it" };   // declare an array of intigers of five elements
    var element = aSequence[3];                               // accessing the element
}
```


#### Sequences

Sequences are linked list, that have a general structure of [head, [tail]], head is the data, and tail is another Linked List. There are many versions of linked list: singular, double, circular etc...

Advantage:

- fast O(1) insertion and removal at any position in the list, as insertion in linked list is only breaking the list, inserting, and repairing it back together (no need to copy the tails)
- linked list is a persistent data structure, rather hard to explain in short sentence, see: wiki-link . This advantage allow tail sharing between two linked list. Tail sharing makes it easy to use linked list as copy-on-write data structure.

Disadvantage:

- Slow O(n) index access (random access), since accessing linked list by index means you have to recursively loop over the list.
- poor locality, the memory used for linked list is scattered around in a mess. In contrast with, arrays which uses a contiguous addresses in memory. Arrays (slightly) benefits from processor caching since they are all near each other

In FOL sequneces are represented like this:
```
seq[type]
```
Example:
```
pro[] main: int = {
    var[new] aSequence: seq[str] = { "get", "over", "it" };   // declare an array of intigers of five elements
    var element = aSequence[3];                               // accessing the element
}
```
### SIMD
Matrixes are of type SIMD (single instruction, multiple data )

#### Matrix
```
mat[sizex]
mat[sizex,sizey]
mat[sizex,sizey,sizez]
```

### Sets
```
set[type,type,type..]
```
A set is a general way of grouping together a number of values with a variety of types into one compound type. Sets have a fixed length: once declared, they cannot grow or shrink in size. In other programming languages they usually are referenced as tuples.
```
pro[] main: int = {
    var aSet: set[str, flt, arr[int, 2]] = { "go", .3, { 0, 5, 3 } };
    var element = aSet[2][1];                                 // accessing the [1] element of the `arr` in the set
    .echo(element)                                            // prints: 5
}
```
### Maps
```
map[key,value]
```
A map is an unordered group of elements of one type, called the element type, indexed by a set of unique keys of another type, called the key type.
```
pro[] main: int = {
    var aMap: map[str, int] = { {"US",45}, {"DE",82}, {"AL",54} };
    var element = aMap["US"];                                 // accessing the "US" key
    .echo(element)                                            // prints: 45
}
```
The number of map elements is called its length. For a map `aMap`, it can be discovered using the built-in function `.len` and may change during execution To add a new element, we use `name+[element]` or `add`function:

```
.echo(.len(aMap))           // prints: 3
aMap.add( {"IT",55} )
aMap+[{"RU",24}]
.echo(.len(aMap))           // prints: 4
```
The comparison operators `==` and `!=` must be fully defined for operands of the key type; thus the key type must not be a function, map, or sequence.

{{% notice tip %}}

Maps are a growable containers too, thus if not allocated in heap but in stack, the size will be defined automatically in compile time and will be changet to static containers

{{% /notice %}}

### Axiom
```
axi[typ, typ]
```
A axiom is a list of facts. A fact is a predicate expression that makes a declarative statement about the problem domain. And whenever a variable occurs in a expression, it is assumed to be universally quantified as [**silent**](/docs/700_sugar/silents/).
```
var likes: axi[str, str] = { {"bob","alice"} , {"alice","bob"}, {"dan","sally"} };
```

{{% notice info %}}

Accesing any container always returns the value, but if we put an `:` before the access symbol so `:[]`, then it will return  `true` or `false` if there is data or not on the specified access.

{{% /notice %}}

```
likes["bob","alice"]                // will return {"bob","alice"}
likes:["bob","alice"]               // will return true
likes["sally","dan"]                // will return {}
likes:["sally","dan"]               // will return false
```

Axioms are a data types that are meant to be used with logic programming. There are containers where facts are stated, and when we want to acces the data, they are always served as containers.

```
var parent: axi[str, str] = { {"albert","bob"}, {"alice","bob"}, {"bob","carl"}, {"bob","tom"} };

parent["bob",*]                     // this gets all elements that "bob" relates to
{"carl", "tom"}
parent[*,"bob"]                     // this gets all elements that "bob" relates from
{"albert", "alice"}
```

Adding new element can be done like in other containers:
```
var parent: axi[str, str] = { {"albert","bob"}, {"alice","bob"}, {"bob","carl"}, {"bob","tom"} };
parent.add({"albert","betty"})
parent.add({"albert","bill"})
parent.add({"alice","betty"})
parent.add({"alice","bill"})
```

And they can be nesetd too:
```
var line: axi[axi[int, int], axi[int, int]] = {{{4,5},{4,8}},{{8,5},{4,5}}}

```

And we can use the simplified form too, just `axi` instead of all the type. We let the compiler fill in the for us:
```
var line: axi = {{{4,5},{4,8}},{{8,5},{4,5}}}

```
