---
title: 'Templates'
type: "docs"
weight: 300
---

Templates are supposed to be mostly used for operator overloading. They are glorified functions, hence used with `pro` or `fun` instead of `def`. 

For example here is how the `!=` is defined: 
```
fun '!='(a, b: int): bol = { return .not(.eq(a, b)) }

.assert( 5 != 4 )
```

or define `$` to return the string version of an object (careful, it is `object$` and not `$object`, the latest is a macro, not a template):
```
pro (file)'$': str = { return "somestring" }

.echo( file$ )
```
