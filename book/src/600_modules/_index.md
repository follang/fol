---
title: "Modules tree"
description: 
draft: false
collapsible: true
weight: 600
---

FOL programs are constructed by linking together packages. A package in turn is constructed from one or more source files that together declare constants, types, variables and functions belonging to the package and which are accessible in all files of the same package. Those elements may be exported and used in another package. 


Every file with extension `.fol` in a folder is part of a package. Thus every file in the folder that uses the same package name, share the same scope between each other. 

Two packages can't exist in same folder, so it is suggested using hierarchy folders to separate packages. 

## Types

Packages can be either:
- defined
- imported
