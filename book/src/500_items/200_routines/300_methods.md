# Methods

There is another type of routine, called method, but it can be either a pure function either a procedure. A method is a piece of code that is called by a name that is associated with an object where it is implicitly passed the object on which it was called and is able to operate on data that is contained within the object.

They either are defined inside the object, or outside the object then the object in which they operate is passed like so (just like in [Golang]()):
```
pro (object)getDir(): str = { result = self.dir; };
```

