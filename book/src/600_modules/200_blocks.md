# Declarations

Each file in a folder (with extension `.fol`) is part of a package. There is no need for imports or other things at the top of the file. They share the same scope, and each declaration is order independent for all files.


## Namespaces

A namespace can be defined in a subfolder of the main foler. And they can be nested.

To acces the namespace there are two ways:
- direct import with `use`
- or code access with `::`

### Direct import

```
use aNS: loc = { "home/folder/printing/logg" }

pro[] main: int = {
    logg.warn("something")
}

```
### Code access

```
use aNS: loc = { "home/folder/printing" }

pro[] main: int = {
    printing::logg.warn("something")
}

```

## Blocks

Block statement is used for scopes where members get destroyed when scope is finished. And there are two ways to define a block: 
- unnamed blocks and 
- named blocks

### Unnamed blocks
Are simply scopes, that may or may not return value, and are represented as: `{ //block }`, with `.` before the brackets for return types and `_` for non return types:
```
pro[] main: int = {
    _{
        .echo("simple type block")
    }
    .echo(.{ return "return type block" })
}
```

### Named blocks
Blocks can be used as labels too, when we want to unconditionally jump to a specific part of the code.
```
pro[] main: int = {
    def block: blk[] = {            // $block A named block that can be referenced
        // implementation
    }
    def mark: blk[]                 // $mark A named block that can be referenced, usually for "jump" statements
}
```


