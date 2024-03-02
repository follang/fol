# Eventuals

Eventuals describe an object that acts as a proxy for a result that is initially unknown, usually because the computation of its value is not yet complete. 

## Async/Await
Async methods are intended to be non-blocking operations. An await expression in an async routine doesn’t block the current thread while the awaited task is running. Instead, the expression signs up the rest of the routine as a continuation and returns control to the caller of the async routine and it means “Once this is done, execute this function”. It’s basically a “when done” hook for your code, and what is happening here is an async routine, when executed, returns a coroutine which can then be awaited. This is done usually in one thread, but can be done in multiple threads too, but thread invocations are invisible to the programmer in this case.
```
pro main(): int = {
    doItFast() | async                                             // compiler knows that this routine has an await routine, thus continue when await rises
    .echo("dosomething to echo")
                                                                   // the main program does not exit until the await is resolved
}
fun doItFast(): str = {
    result = client.get(address).send() | await                    // this tells the routine that it might take time
    .echo(result)
}
```
