---
title: 'Alternatives'
type: "docs"
weight: 250
---

Alternatives are used when we want to simplify code. For example, define an alternative, so whenever you write `+var` it is the same as `var[+]`.
```
def '+var': alt = 'var[+]'
def '~var': alt = 'var[~]'
def '.pointer_content': alt = '.pointer_value'
```
