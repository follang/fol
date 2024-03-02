---
title: 'Limits'
type: "docs"
weight: 400
---

Limiting is a syntactic way to set boundaries for variables. The way FOL does is by using `[]` right after the type declaration `type[]`, so: `type[options][limits]`


## Initger limiting
Example, making a intiger variable have only numbers from 0 to 255 that represents an RGB value for a single color:
```
var rgb: int[][.range(255)];

```

## Character limiting
It works with strings too, say we want a string that can should be of a particular form, for example an email:

```
var email: str[][.regex('[._a-z0-9]+@[a-z.]+')]
```


