---
title: 'Chaining'
type: "docs"
weight: 800
---

Optional chaining is a process for querying and calling properties, methods, and subscripts on an optional that might currently be nil. If the optional contains a value, the property, method, or subscript call succeeds; if the optional is nil, the property, method, or subscript call returns nil. Multiple queries can be chained together, and the entire chain fails gracefully if any link in the chain is nil.

Before I can really explain optional chaining, we have to get a grasp of what an optional value is. In FOL, variables cannot be empty. In other words, variables cannot store a value of NIL, at least not directly. This is a great feature because we can assume that all variables contain some value. Of course, sometimes variables need to be NIL. Fortunately, FOL provides that through a boxing feature called optionals. Optionals allow a user to wrap a value in a container which can be unwrapped to reveal either a value or NIL:

    var printString: ?str;
    printString = "Hello, World!"
    .echo(printString!)

In this example, we declare an optional string and give it a value of “Hello, World!” Since we know that the variable stores a `str`, we can unconditionally unwrap the value and echo it. Of course, unconditional unwrapping is typically bad practice, so I’m only showing it for the purposes of showing off optionals.

At any rate, optional chaining takes this concept of optionals and applies it to method calls and fields. For instance, imagine we have some long chain of method calls:

    important_char = commandline_input.split('-').get(5).charAt(7)

In this example, we take some command line input and split it by hyphen. Then, we grab the fifth token and pull out the seventh character. If at any point, one of these method calls fails, our program will crash.

With optional chaining, we can actually catch the NIL return values at any point in the chain and fail gracefully. Instead of a crash, we get an important_char value of NIL. Now, that’s quite a bit more desirable than dealing with the pyramid of doom.
