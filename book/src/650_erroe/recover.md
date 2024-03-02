---
title: 'Recover'
type: "docs"
weight: 200
---

`report` can be used to handle recoverable errors. As [discussed here](/docs/spec/functions/#return), FOL uses two variables `result` nd `error` in return of each routine. As name implies, `result` represents the type of the value that will be returned in a success case, and `error` represents the type of the error `err[]` that will be returned in a failure case.

When we use the keyword `report`, the error is returned to the routine's error variable and the routine qutis executing (the routine, not the program).
```
use file: mod[std] = { std::fs::File }

pro main(): int = {
    pro[] fileReader(path: str): str = {
        var aFile = file.readfile(path)
        if ( check(aFile) ) {
            report "File could not be opened" + file                        // report will not break the program, but will return the error here, and the routine will stop
        } else {
            return file.to_string()                                         // this will be executed only if file was oopened without error
        }
    }
}
```

Form this point on, the error is concatinated up to the main function. This is known as propagating the error and gives more control to the calling code, where there might be more information or logic that dictates how the error should be handled than what you have available in the context of your code.

```
use file: mod[std] = { std::fs::File }

pro main(): int = {
    var f = file.open("main.jpg");                                           // main.jpg doesn't exist
    if (check(f)) {
        report "File could not be opened" + file                             // report will not break the program
    } else {
        .echo("File was open sucessfulley")                                  // this will be executed only if file was oopened without error
    }
}
```
