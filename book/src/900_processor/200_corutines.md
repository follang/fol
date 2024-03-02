# Coroutines

A coroutine is a task given form the main thread, similar to a routine, that can be in concurrent execution with other tasks of the same program though other routines.  A **worker** takes the task and runs it, concurrently. Each task in a program can be assigned to one or multiple workers. 

Three characteristics of coroutine distinguish them from normal routines:
- First, a task may be implicitly started, whereas a routine must be explicitly called. 
- Second, when a program unit invokes a task, in some cases it need not wait for the task to complete its execution before continuing its own. 
- Third, when the execution of a task is completed, control may or may not return to the unit that started that execution.
- Fourth and most importantly, the execution of the routine is entirely independent from main thread.

In fol to assign a task to a worker, we use the symbols `[>]` 

## Channels

FOL provides asynchronous channels for communication between threads. Channels allow a unidirectional flow of information between two end-points: the Transmitter and the Receiver. It creates a new asynchronous channel, returning the tx/tx halves. All data sent on the Tx (transmitter) will become available on the Rx (receiver) in the same order as it was sent. The data is sent in a sequence of a specifies type `seq[type]`. `tx` will not block the calling thread while `rx` will block until a message is available.

```
pro main(): int = {
    var channel: chn[str];
    for (0 ... 4) {
        [>]doItFast() | channel[tx]                                             // sending the output of four routines to a channel transmitter
                                                                                // each transmitter at the end sends the close signal
    }
    var fromCh1 = channel[rx][0]                                                // reciveing data from one transmitter, `0`
}

fun doItFast(i: int; found: bol): str = {
    return "hello"
}
```

If we want to use the channel within the function, we have to clone the channel's tx and capture with an ananymus routine: Once the channels transmitter goes out of scope, it gets disconnected too.
```
pro main(): int = {
    var channel: chn[str];                                                      // a channel with four buffer transmitters
    var sequence: seq[str];

    for (0 ... 4) {
        [>]fun()[channel[tx]] = {                                               // capturin gthe pipe tx from four coroutines
            for(0 ... 4){
                "hello" | channel[tx]                                           // the result are sent fom withing the funciton eight times
            }
        }                                                                       // when out of scope a signal to close the `tx` is sent
    }

    select(channel as c){
        sequence.push(channel[rx][c])                                           // select statement will check for errors and check which routine is sending data
    }
}
```

## Locks - Mutex

Mutex is a locking mechanism that makes sure only one task can acquire the mutexed varaible at a time and enter the critical section. This task only releases the mutex when it exits the critical section. It is a mutual exclusion object that synchronizes access to a resource. 

In FOL mutexes can be passed only through a routine. When declaring a routine, instead of using the borrow form with `( // borrowing variable )`, we use double brackets `(( // mutex ))`. When we expect a mutex, then that variable, in turn has two method more: 

- the `lock()` which unwraps the variable from mutex and locks it for writing and 
- the `unlock()` which releases the lock and makes the file avaliable to other tasks
```

fun loadMesh(path: str, ((meshes)): vec[mesh]) = {                              // declaring a *mutex and *atomic reference counter with double "(( //declaration ))"
    var aMesh: mesh = mesh.loadMesh(path) 
    meshes.lock()
    meshes.push(aMesh)                                                          // there is no need to unlock(), FOL automatically drops at the end of funciton
                                                                                // if the function is longer, then we can unlock to not keep other tasks waiting
}

pro main(): int = {
    ~var meshPath: vec[str];
    ~var meshes: vec[mesh];
    var aFile = file.readfile(filepath) || .panic("cant open the file")

    each( line in aFile.line() ) { meshPath.push(line) };
    for(m in meshPath) { [>]loadMesh(m, meshes) };
}
```

