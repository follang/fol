# Pointers

The only way to access the same memory with different variable is by using pointers. In example below, we create a pointer, and when we want to dereference it to modify the content of the address that the pointer is pointing to, we use `*ptrname` or `.pointer_value(ptrname)`.
```
@var aContainer: arr[int, 5];                                    //allocating memory on the heap
var contPoint: ptr[] = aContainer;
    
*contPoint = { zero, one, two, three, four };                    //dereferencing and then assigning values
```
Bare in mind, that the pointer (so, the address itself) can't be changes, unless when created is marked as `var[mut]`. To see tha address of a pointer we use `&ptrname` or `.address_of(ptrname)`
```
@var aContainer: arr[int, 5];                                    //allocating memory on the heap
var contPoint: ptr[] = aContainer;
    
var anotherPoint: ptr[] = &contPoint;                            //assigning the same adress to another pointer
```
## Unique pointer

Ponter of a pointer is very simimilar to RUST move pointer, it actually, deletes the first pointer and references the new one to the location of deleted one. However this works only when the pointer is unique (all pointers by default all unique). This is like borrowing, but does not invalidate the source variable:
```
var aContainer: arr[int, 5] = { zero, one, two, three, four };

var contPoint: ptr[] = aContainer;
var anotherPoint: ptr[] = &contPoint;
```

with borrowing, we use `#varname` or `.borrow_from(varname)`
```
var aContainer: arr[int, 5] = { zero, one, two, three, four };
{
    var borrowVar = #aContainer;                                    //this makes a new var form the old var, but makes the old invalid (until out of scope)
}
```

## Shred pointer
Ponter can be shared too. They can get referenced by another pointer, and they don't get destroyed until the last reference's scope is finished. This is exacly like smart shared_ptr in C++. Pointer to this pointer makes a reference not a copy as unique pointers. Dereferencing is a bit complicated here, as when you dereference a pointer pointer you get a pointer, so you need to dereference it too to get the value.
```
@var aContainer: arr[int, 5] = { zero, one, two, three, four };

var contPoint: ptr[] = aContainer;
var pointerPoint: ptr[shared] = &contPoint;
```
Dereferencing (step-by-step):
```
var apointerValue = *pointerPoint
var lastpointerValue = *apointer
```
Dereferencing (all-in-one):
```
var lastpointer = *(*pointerPoint)
```
## Raw pointer
Lastly, pointers can be raw too. This is the base of ALL POINTERS AND VARIABLES. Pointers of this type need to MANUALLY GET DELETED. If a pointer gets deleted before the new pointer that points at it, we get can get memory corruptions:
```
var aContainer: arr[int, 5] = { zero, one, two, three, four };

var contPoint: ptr[raw] = aContainer;
var pointerPoint: ptr[raw] = &contPoint;
```
Deleting:
```
!(pointerPoint)
!(contPoint)
```
