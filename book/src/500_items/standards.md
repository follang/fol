---
title: 'Standards'
type: "docs"
weight: 65
---

## Satndard

A standard is an established norm or requirement for a repeatable technical task. It is usually a formal declaration that establishes uniform technical criteria, methods, processes, and practices. 

S, what is a to be considered a standard:

- A standard specification is an explicit set of requirements for an item, object or service. It is often used to formalize the technical aspects of a procurement agreement or contract. 
- A standard test method describes a definitive procedure that produces a test result. It may involve making a careful personal observation or conducting a highly technical measurement. 
- A standard procedure gives a set of instructions for performing operations or functions.
- A standard guide is general information or options that do not require a specific course of action.
- A standard definition is formally established terminology.


In FOL, standards are named collection of method signatures and are created by using `std` keyword:
```
typ geometry = {
    fun area(): flt[64];
    fun perim(): flt[64];
};
```

There are three types of standards, 

- protocol `pro[]` that enforce just function implementation
- blueprint `blu[]` that enforces just data implementation
- extended `ext[]`, that enforces function and data:
```
std geometry: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};


std geometry: blu = {
    var color: rgb; 
    var size: int;
};

std geometry: ext = {
    fun area(): flt[64];
    fun perim(): flt[64];
    var color: rgb;
    var size: int;
};
```
## Contract
A contract is a legally binding agreement that recognises and governs the rights and duties of the parties to the agreement. A contract is enforceable because it meets the requirements and approval of an higher authority. An agreement typically involves a written declaration given in exchange for something of value that binds the maker to do. Its an specific act which gives to the person to whom the declaration is made the right to expect and enforce performance. In the event of breach of contract, the higher authority will refrain the contract from acting.

In fol contracts are used to bind a type to a standard. If a type declares to use a standard, it is the job of the contract (compiler internally) to see the standard full-filled.

```
std geo: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};

std rect(geo): rec[] = {                                             // this type makes a contract to use the geometry standard
    width: int[64];
    heigh: int[64];
}

```
Now we can make `rect` records or classes, we have to respect the contract. If we don't implement the `geo` methods, when we instantiate a new object of type `rect` it will throw an error.
```
var aRectangle: rect = { width = 5, heigh = 6 }                      // this throws an error, we haven't fullfill the ocntract
```

To do so, we need first to create the default `rect` methods from `geo` standard, then instantiate a new object:

```
fun (rect)area(): flt[64] = { result = self.width + self.heigh }
fun (rect)perim(): flt[64] = { result = 2 * self.width + 2 * self.heigh }

var aRectangle: rect = { width = 5, heigh = 6 }                     // this from here on will work
```

The benifit of standard is that, we can create a routine that as parameter takes a standard, thus all objects with the standard can use afterwards that routine:

```
std geo: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};

typ rect(geo): rec[] = {                                            // this type makes a contract to use the geometry standard
    width: int[64]; 
    heigh: int[64]; 
}
fun (rect)area(): flt[64] = { result = self.width + self.heigh }
fun (rect)perim(): flt[64] = { result = 2 * self.width + 2 * self.heigh }

typ circle(geo): rec[] = {                                          // another type makes a contract to use the geometry standard
    radius: int[64]; 
}
fun (circle)area(): flt[64] = { result = math::const.pi * self.radius ** 2 }
fun (circle)perim(): flt[64] = { result = 2 * math::const.pi * self.radius}

typ square: rec[] = {                                               // this type does not make contract with `geo`
    heigh: int[64] 
}

pro measure( shape: geo) { .echo(shape.area() + "m2") }        // a siple method to print the standard's area

// instantiate two objects
var aRectangle: rect = { width = 5, heigh = 6 }                      // creating a new rectangle
var aCircle: circle = { radius = 5 }                                 // creating a new rectangle
var aSquare: square = { heigh = 6 }                                  // creating a new square


// to call the measure function that rpints the surface
measure(aRectangle)                                                  // this prints: 30m2
measure(aSquare)                                                     // this throws error, square cant use measure method
measure(aCircle)                                                     // this prints: 78m2

```
