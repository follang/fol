---
title: 'Defaults'
type: "docs"
weight: 260
---

Defaults are a way to change the default behaviour of options. Example the default behaviour of `str` when called without options. By defalt `str` is it is saved on stack, it is a constant and not public, thus has `str[pil,imu,nor]`, and we want to make it mutable and saved on heap by default:
```
def 'str': def[] = 'str[new,mut,nor]'
```

