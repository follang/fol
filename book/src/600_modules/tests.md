---
title: 'Tests'
type: "docs"
weight: 50
---

Blocks defined with type `tst`, have access to the module (or namespace) defined in `tst["name", access]`.

```
def test1: tst["sometest", shko] = {}
def "some unit testing": tst[shko] = {}
```

