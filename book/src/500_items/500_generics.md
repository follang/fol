# Generics

## Types
### Generic functions - lifting

The generic programming process focuses on finding commonality among similar implementations of the same algorithm, then providing suitable abstractions so that a single, generic algorithm can cover many concrete implementations. This process, called lifting, is repeated until the generic algorithm has reached a suitable level of abstraction, where it provides maximal reusability while still yielding efficient, concrete implementations. The abstractions themselves are expressed as requirements on the parameters to the generic algorithm.


```
pro max[T: gen](a, b: T): T = {
	result =  a | a < b | b;
};
fun biggerFloat(a, b: flt[32]): flt[32] = { max(a, b) }
fun biggerInteger(a, b: int[64]): int[64] = { max(a, b) }
```

### Generic types - concepts

Once many algorithms within a given problem domain have been lifted, we start to see patterns among the requirements. It is common for the same set of requirements to be required by several different algorithms. When this occurs, each set of requirements is bundled into a concept. A concept contains a set of requirements that describe a family of abstractions, typically data types. Examples of concepts include Input Iterator, Graph, and Equality Comparable. When the generic programming process is carefully followed, the concepts that emerge tend to describe the abstractions within the problem domain in some logical way.

```
typ container[T: gen, N: int](): obj = {
	var anarray: arr[T,N];
	+fun getsize(): num = { result = N; }
};
var aContainer: container[int, 5] = { anarray = {zero, one, two, three, four}; };
```

## Dispach

Static dispatch (or early binding) happens when compiler knows at compile time which function body will be executed when I call a method. In contrast, dynamic dispatch (or run-time dispatch or virtual method call or late binding) happens when compiler defers that decision to run time. This runtime dispatch requires either an indirect call through a function pointer, or a name-based method lookup.

```
std foo: pro = { fun bar(); }

typ[ext] int, str: int, str;

fun (int)bar() = {  }
fun (str)bar() = {  }

pro callBar(T: foo)(value: T) = { value.bar() }             // dispatch with generics
pro barCall( value: foo ) = { value.bar() }                 // dispatch with standards

pro main: int = {
    callBar(2);
    callBar("go");

    barCall(2);
    barCall("go")
}
```
