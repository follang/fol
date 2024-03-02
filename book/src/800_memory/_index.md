# Memory model

## The Stack

What is the stack? It's a special region of your computer's memory that stores temporary variables created by each function (including the main() function). The stack is a "LIFO" (last in, first out) data structure, that is managed and optimized by the CPU quite closely. Every time a function declares a new variable, it is "pushed" onto the stack. Then every time a function exits, all of the variables pushed onto the stack by that function, are freed (that is to say, they are deleted). Once a stack variable is freed, that region of memory becomes available for other stack variables.

The advantage of using the stack to store variables, is that memory is managed for you. You don't have to allocate memory by hand, or free it once you don't need it any more. What's more, because the CPU organizes stack memory so efficiently, reading from and writing to stack variables is very fast.

A key to understanding the stack is the notion that when a function exits, all of its variables are popped off of the stack (and hence lost forever). Thus stack variables are local in nature. This is related to a concept we saw earlier known as variable scope, or local vs global variables. A common bug is attempting to access a variable that was created on the stack inside some function, from a place in your program outside of that function (i.e. after that function has exited).

Another feature of the stack to keep in mind, is that there is a limit (varies with OS) on the size of variables that can be stored on the stack. This is not the case for variables allocated on the heap.

- very fast access
- don't have to explicitly deallocate variables
- space is managed efficiently by CPU (memory will not become fragmented)
- local variables only
- limit on stack size (OS-dependent)
- variables cannot be resized


## The Heap

The heap is a region of your computer's memory that is not managed automatically for you, and is not as tightly managed by the CPU. It is a more free-floating region of memory (and is larger). To allocate memory on the heap, you must use `var[new]`. Once you have allocated memory on the heap, you are responsible to deallocate that memory once you don't need it any more. If you fail to do this, your program will have what is known as a memory leak. That is, memory on the heap will still be set aside (and won't be available to other processes).

Unlike the stack, the heap does not have size restrictions on variable size (apart from the obvious physical limitations of your computer). Heap memory is slightly slower to be read from and written to, because one has to use pointers to access memory on the heap. Unlike the stack, variables created on the heap are accessible by any function, anywhere in your program. Heap variables are essentially global in scope.

Element of the heap have no dependencies with each other and can always be accessed randomly at any time. You can allocate a block at any time and free it at any time. This makes it much more complex to keep track of which parts of the heap are allocated or free at any given time.

- variables can be accessed globally
- no limit on memory size
- slower access
- no guaranteed efficient use of space, memory may become fragmented over time as blocks of memory are allocated, then freed
- you must manage memory (you're in charge of allocating and freeing variables)
- variables can be resized anytime

## Multithread
In a multi-threaded situation each thread will have its own completely independent stack but they will share the heap. Stack is thread specific and Heap is application specific. The stack is important to consider in exception handling and thread executions.

## Memory and Allocation

In the case of a normal variable, we know the contents at compile time, so the value is hardcoded directly into the final executable. This is why they are fast and efficient. But these properties only come from the variable immutability. Unfortunately, we can’t put a blob of memory into the binary for each piece of variable whose size is unknown at compile time and whose size might change while running the program.

Lets take an example, the user input as `str` (string), in order to support a mutable, growable piece of variable, we need to allocate an amount of memory on the heap, unknown at compile time, to hold the contents. This means:

- The memory must be requested from the operating system at runtime.
- We need a way of returning this memory to the operating system when we’re done with our String.

That first part is done by us: when we call `var[new]`, its implementation requests the memory it needs. This is pretty much universal in any programming languages.

However, the second part is different. In languages with a garbage collector (GC), the GC keeps track and cleans up memory that isn’t being used anymore, and we don’t need to think about it. Without a GC, it’s our responsibility to identify when memory is no longer being used and call code to explicitly return it, just as we did to request it. Doing this correctly has historically been a difficult programming problem. If we forget, we’ll waste memory. If we do it too early, we’ll have an invalid variable. If we do it twice, that’s a bug too. We need to pair exactly one allocate with exactly one free.

Fol, copies [Rust](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) in this aspect: the memory is automatically returned once the variable that owns it goes out of scope. 

{{% notice warn %}}

When a variable goes out of scope, Fol calls a special function for us to deallocate all the new memories we have allocated during this dunction call. 

{{% /notice %}}


This function is called `.de_alloc()` - just like Rust's `drop()`, and it’s where the author of `var[new]` can put the code to return the memory. Fol calls `.de_alloc()` automatically at the closing curly bracket.
