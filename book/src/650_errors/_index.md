# Error handling


Unlike other programming languages, **FOL** does not have exceptions ([Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html) neither). It has only two types of errors:
- braking errors
- recoverable errors

Breaking errors cause a program to fail abruptly. A program cannot revert to its normal state if an unrecoverable error occurs. It cannot retry the failed operation or undo the error. An example of an unrecoverable error is trying to access a location beyond the end of an array.

Recoverable error are errors that can be corrected. A program can retry the failed operation or specify an alternate course of action when it encounters a recoverable error. Recoverable errors do not cause a program to fail abruptly. An example of a recoverable error is when a file is not found.


There are two keywords reserved and associated to two types of errors: `report` for a recoverable error and `panic` for the braking error. 


{{% notice tip %}}

By default, all errors are either conctinated up with report, or exited with panic.

{{% /notice %}}

A simplier way to hande errors is through [pipes](/docs/spec/pipes)
