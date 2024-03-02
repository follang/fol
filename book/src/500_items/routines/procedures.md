---
title: 'Procedures'
type: "docs"
weight: 10
---

Procedures are most common type of routines in Fol. When a procedure is "called" the program "leaves" the current section of code and begins to execute the first line inside the procedure. Thus the procedure "flow of control" is:

- The program comes to a line of code containing a "procedure call".
- The program enters the procedure (starts at the first line in the procedure code).
- All instructions inside of the procedure are executed from top to bottom.
- The program leaves the procedure and goes back to where it started from.
- Any data computed and RETURNED by the procedure is used in place of the procedure in the original line of code.

Procedures have side-effects, it can modifies some state variable value(s) outside its local environment, that is to say has an observable effect besides returning a value (the main effect) to the invoker of the operation. State data updated "outside" of the operation may be maintained "inside" a stateful object or a wider stateful system within which the operation is performed.

### Passing values

The semantics for passing a value to a procedure are similar to those for assigning a value to a variable. Passing a variable to a procedure will move or copy, just as assignment does. If the procedure is stack-based, it will automatically copy the value. If it is heap-based, it will move the value. 
```
pro[] modifyValue(someStr: str) = {
    someStr = someStr + " world!"
}

pro[] main: int =  {
                                        //case1
    var[mut] aString: str = "hello";                        // a string varibale $aString is declared (in stack as default)
    modifyValue(aString);                                   // the value is passed to a procedure, since $aVar is in stack, the value is copied
    .echo(aString)                                          // this prints: "hello", 
                                                            // value is not changed and still exists here, because was copied

                                        //case2
    @var[mut] aString: str = "hello";                       // a string varibale $bString is declared (in stack with '@')
    modifyValue(bString);                                   // the value is passed to a procedure, since $aVar is in heap, the value is moved
    .echo(bString)                                          // this throws ERROR, 
                                                            // value does not exists anymore since it moved and ownership wasn't return
}
```
As you can see from above, in both cases, the `.echo(varable)` does not reach the desired procedure, to print `hello world!`. In first case is not changed (because is coped), in second case is changed but never returned. To fix the second case, we can just use the `.give_back()` procedure to return the ownership:
```
pro[] modifyValue(someStr: str) = {
    someStr = someStr + " world!"
    .give_back(someStr)                                     // this returns the ownership (if there is an owner, if not just ignores it)
}

pro[] main: int =  {
                                        //case1
    var[mut] aString: str = "hello";                        // a string varibale $aString is declared (in stack as default)
    modifyValue(aString);                                   // the value is passed to a procedure, since $aVar is in stack, the value is copied
    .echo(aString)                                          // this still prints: "hello", 
                                                            // value is not changed and still exists here, because was copied

                                        //case2
    @var[mut] aString: str = "hello";                       // a string varibale $bString is declared (in stack with '@')
    modifyValue(bString);                                   // the value is passed to a procedure, since $aVar is in heap, the value is moved
    .echo(aString)                                          // this now prints: "hello world!", 
                                                            // value now exists since the ownership is return
}
```
### Lend parameters

But now, we were able to change just the variable that is defined in heap (case two), by moving back the ownership. In case one, since the value is copied, the owner of newly copied value is the procedure itself. So the `.give_back()` is ignored. To fix this, we use [borrowing](/docs/spec/050_pointers/#borrowing) to lend a value to the procedure
```
pro[] modifyValue(SOMESTR: str) = {                         // we use allcaps `SOMESTR` to mark it as borrowable
    somestr = someStr + " world!"                           // when we refer, we can both refer with ALLCAPS or lowecaps
}

pro[] main: int =  {
                                        //case1
    var[mut] aString: str = "hello";                        // a string varibale $aString is declared (in stack as default)
    modifyValue(aString);                                   // the value is lended to the procedure
    .echo(aString)                                          // this now prints: "hello world!", 

                                        //case2
    @var[mut] aString: str = "hello";                       // a string varibale $bString is declared (in heap with '@')
    modifyValue(aString);                                   // the value is lended to the procedure
    .echo(aString)                                          // this now prints: "hello world!", 
}
```
{{% notice warn %}}

So to make a procedure borrow a varibale it uses all caps name `A_VAR`. 
Remember that two variables are the same if have same characters (does not matter the caps)

{{% /notice %}}
```
pro[] borrowingProcedure(aVar: str; BVAR: bol; cVar, DVAR: int)
```

To call this procedure, the borrowed parameters always shoud be a variable name and not a direct value:

```
var aBool, anInt = true, 5
borrowingProcedure("get", true, 4, 5)                        // this will throw an error, cos it expects borrowable not direct value
borrowingProcedure("get", aBool, 4, anInt)                   // this is the proper way

```

When the value is passed as borrowable in procedure, by default it gives premission to change, so the same as `var[mut, bor]` as [disscussed here](/docs/spec/050_pointers/#borrowing).

### Return ownership

Return values can be though as return of ownership too. The ownership of a variable follows the same pattern every time: assigning a value to another variable moves or copies it. 
```
pro main(): int = {
    var s1 = givesOwnership();                              // the variable $s1 is given the ownership of the procedure's $givesOwnership return
    .echo(s1)                                               // prints "hi"
    var s2 = returnACopy();                                 // the variable $s2 is given the ownership of the procedure's $returnACopy return
    .echo(s2)                                               // prints: "there"
}
pro givesOwnership(): str = {                               // This procedure will move its return value into the procedure that calls it
    @var someString = "hi";                                 // $someString comes into scope
    return someString                                       // $someString is returned and MOVES out to the calling procedure
}
pro returnACopy(): int = {                                  // This procedure will move its return value into the procedure that calls it
    var anotherString = "there"                             // $anotherString comes into scope
    return anotherString                                    // $anotherString is returned and COPIES out to the calling procedure
}
```
When a variable that includes data on the heap goes out of scope, the value will be cleaned up automatically by `.de_alloc()` unless the data has been moved to be owned by another variable, in this case we give the ownership to return value. If the procedure with the retun value is not assigned to a variable, the memory will be freed again.

We can even do a transfer of ownership by using this logic:
```
pro main(): int = {
    @var s2 = "hi";                                         // $s2 comes into scope (allocatd in the heap)
    var s3 = transferOwnership(s2);                         // $s2 is moved into $transferOwnership procedure, which also gives its return ownership to $s3
    .echo(s3)                                               // prints: "hi"
    .echo(s2)                                               // this throws an error, $s2 is not the owner of anything anymore
}

pro transferOwnership(aString: str): str = {                // $aString comes into scope
    return aString                                          // $aString is returned and moves out to the calling procedure
}
```

This does not work with borrowing though. When a variable is lended to a procedure, it has permissions to change, but not lend to someone else. The only thing it can do is make a `.deep_copy()` of it:
```
pro main(): int = {
    @var s2 = "hi";                                         // $s2 comes into scope (allocatd in the heap)
    var s3 = transferOwnership(s2);                         // $s2 is moved into $transferOwnership procedure, which also gives its return ownership to $s3
    .echo(s3)                                               // prints: "hi"
    .echo(s2)                                               // prints: "hi" too
}

pro transferOwnership((aString: str)): str = {              // $aString comes into scope which is borrowed
    return aString                                          // $aString is borrowed, thus cant be lended to someone else
                                                            // thus, the return is a deep_copy() of $aString
}
```
