---
title: 'Pipes'
type: "docs"
weight: 200
---

Piping is a process that connects the output of the expression to the left to the input of the expression of the right. You can think of it as a dedicated program that takes care of copying everything that one expressionm prints, and feeding it to the next expression. The idea is the same as `bash pipes`. For example, an routine output is piped to a conditional through pipe symbol `|` then the conditional takes the input and returns true or false. If returned false, then the second part of pipe is returned. To access the piped variable, `this` keyword is used:
```
pro[] main: int = {
    fun addFunc(x, y: int): int = {
        return x + y;
    }
    var aVar: int = addFunc(4, 5) | if(this > 8) | return 6;
}
```
However, when we assign an output of a function to a variable, we shoud expect that errors within funciton can happen. By default, everytime a function is called, and the function throws an error in will be reported up.
```
var aVar: int = addFunc(4, 5);                                  // if there are errors, and the call is in main function, the program will exit
                                                                // because is the last concatinator of the 'report' error
```

However, when we use pipes, we pass the function values (result and the error) to the next expression, and then, it is the second expression's responsibility to deal with it. We use the built-in `check` that checks for error on the function:
```
var aVar: int = addFunc(4, 5) | check(this) | return 5;         // if there are errors, the error is passed to the next sepression with pipe
                                                                // here, if there is errors, will be checked and the default value of 5 will return
```

There is a shorter way to do this kind of error checking. For that we use double pipe `||`. For example, we assign the output of a function to a variable, but the function may fail, so we want a default variable:
```
var aVar: int = addFunc(4, 5) || return 5;
```

Or to handle the error ourselves. This simply says, if i get the error, then we can `panic` or `report` with custom message:
```
var aVar: int = addFunc(4, 5) || panic "something bad inside function has happened";
```

More on error handling can be [found here](/docs/spec/errors) 


