---
title: 'Structs'
type: "docs"
weight: 60
---

Structs are the way to declare new type of data. A struct binds an identifier, the type name, to a type.

A struct definition creates a new, distinct type and are few of them in FOL:

- records
- entries

## Definition

### Records

A record is an aggregate of data elements in which the individual elements are identified by names and types and accessed through offsets from the beginning of the structure. There is frequently a need in programs to model a collection of data in which the individual elements are not of the same type or size. For example, information about a college student might include name, student number, grade point average, and so forth. A data type for such a collection might use a character string for the name, an integer for the student number, a  floating- point for the grade point average, and so forth. Records are designed for this kind of need.

It may appear that records and heterogeneous [set](/docs/spec/types/#sets) are the same, but that is not the case. The elements of a heterogeneous `set[]` are all references to data objects that reside in scattered locations, often on the heap. The elements of a record are of potentially different sizes and reside in adjacent memory locations. Records are normally used as encapsulation structures, rather than data structures. 
```
typ user: rec = {
    var username: str;
    var email: str;
    var sign_in_count: int[64];
    var active: bol;
};
```

#### Records as classes
Calsses are the way that FOL can apply OOP paradigm. They basically are a glorified record. Instead of methods to be used fom outside the body, they have the method declaration within the body. For example, creating an class `computer` and its methods within the body:
```
~typ[pub] computer: rec = {
    var[pub] brand: str;
    var[pub] memory: int[16];

    +fun getType(): str = { brand + .to_string(memory) };
};

var laptop: computer = { member1 = value, member2 = value };
.echo(laptop.getType());
```



### Entries

Is an a group of constants (identified with `ent`) consisting of a set of named values called elements.
```
typ color: ent = {
    var BLUE: str = "#0037cd" 
    var RED str = "#ff0000" 
    var BLACK str = "#000000" 
    var WHITE str = "#ffffff" 
};

if( something == color.BLUE ) { doathing } else { donothing }
```

#### Entries as enums
Unums represent enumerated data. An enumeration type (or enum type) is a value type defined by a set of named constants of the underlying integral numeric type.
```
typ aUnion: ent = {
    var BLUE, RED, BLACK, WHITE: int[8] = {..3}
}
```

## Initializaion

To use a record after we’ve defined it, we create an instance of that record by specifying concrete values for each of the fields. We create an instance by stating the name of the record and then add curly brackets containing key: value pairs, where the keys are the names of the fields and the values are the data we want to store in those fields. We don’t have to specify the fields in the same order in which we declared them in the record. In other words, the record definition is like a general template for the type, and instances fill in that template with particular data to create values of the type.
```
@var user1: user = {
    email = "someone@example.com",
    username = "someusername123",
    active = true,
    sign_in_count = 1,
};
```

### Named initialization:
```
@var[mut] user1: user = { email = "someone@example.com", username = "someusername123", active = true, sign_in_count = 1 }
```

### Ordered initialization
```
@var[mut] user1: user = { "someone@example.com", "someusername123", true, 1 }
```


## Accessing

To get a specific value from a record, we can use dot notation or the access brackets. If we wanted just this user’s email address, we could use `user1.email` or `user1[email]` wherever we wanted to use this value. If the instance is mutable, we can change a value by assigning into a particular field. Note that the entire instance must be mutable; FOL doesn’t allow us to mark only certain fields as mutable. 
```
@var[mut] user1: user = {
    email = "someone@example.com",
    username = "someusername123",
    active = true,
    sign_in_count = 1,
};

user1.email = "new.mail@example.com"
user1[username] = "anotherusername"
```
## Returning

As with any expression, we can construct a new instance of the record as the last expression in the function body to implicitly return that new instance. As specified [in function return](/docs/spec/functions/#return), the final expression in the function will be used as return value. For this to be used, the return type of the function needs to be defined (here is defined as `user`) and this can be used only in one statement body. Here we have declared only one variable `user1` and that itslef spanc into multi rows:
```
pro buildUser(email, username: str): user = { user1: user = {
    email = "someone@example.com",
    username = "someusername123",
    active = true,
    sign_in_count = 1,
} }
```
## Nesting

Records can be nested by creating a record type using other record types as the type for the fields of record. Nesting one record within another can be a useful way to model more complex structures: 
```
var empl1: employee = {
    FirstName = "Mark",
    LastName =  "Jones",
    Email =     "mark@gmail.com",
    Age =       25,
    MonthlySalary = {
        Basic = 15000.00,
        Bonus = {
            HTA =    2100.00,
            RA =   5000.00,
        },
    },
}
```

## Defauling

Records can have default values in their fields too. 
```
typ user: rec = {
    var username: str;
    var email: str;
    var sign_in_count: int[64] = 1;
    var active: bol = true;
};
```

This makes possible to enforce some fields (empty ones), and leave the defaults untouched: 
```
@var[mut] user1: user = { email = "someone@example.com", username = "someusername123" }

```
## Limiting

We can also restrict the values (with ranges) assigned to each field:
```
typ rgb: rec[] = {
    var r: int[8][.range(255)];
    var g: int[8][.range(255)];
    var b: int[8][.range(255)];
}

var mint: rgb = { 153, 255, 187 }
```

This of course can be achieve just with variable types and aliased types and sets too, but we would need to create two types:
```
typ rgb: set[int[8][.range(255)], int[8][.range(255)], int[8][.range(255)]];

var mint: rgb = { 153, 255, 187 }
```
## Methods

A record may have methods associated with it. It does not inherit any methods bound to the given type, but the method set of an standard type remains unchanged.To create a method for a record, it needs to be declared as the reciever of that method, in FOL's. Making a getter fucntion:
```
fun (recieverRecord)someFunction(): str = { self.somestring; };
```

After declaring the record receiver, we then we have access to the record with the keyword `self`. A receiver is essentially just a type that can directly call the method. 
```
typ user: rec = {
    var username: str;
    var email: str;
    var sign_in_count: int[64];
    var active: bol;
};

fun (user)getName(): str = { result = self.username; };
```

Methods have some benefits over regular routines. In the same package routines with the same name are not allowed but the same is not true for a method. One can have multiple methods with the same name given that the receivers they have are different. 

Then each instantiation of the record can access the method. Receivers allow us to write method calls in an OOP manner. That means whenever an object of some type is created that type can call the method from itself.
```
var[mut] user1: user = { email = "someone@example.com", username = "someusername123", active = true, sign_in_count = 1 }

.echo(user1.getName());
```

