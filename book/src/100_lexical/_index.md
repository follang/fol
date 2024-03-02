---
title: "Lexical analysis"
description: 
draft: false
collapsible: true
weight: 100
---

A lexical analyzer is essentially a pattern matcher. A pattern matcher attempts to find a substring of a given string of characters that matches a given character pattern. All FOL's input is interpreted as a sequence of **UNICODE** code points encoded in UTF-8. A lexical analyzer serves as the front end of a syntax analyzer. Technically, lexical analysis is a part of syntax analysis. A lexical analyzer performs syntax analysis at the lowest level of program structure. An input program appears to a compiler as a single string of characters. The lexical analyzer collects characters into logical groupings and assigns internal codes to the groupings according to their structure.

Syntax analyzers, or parsers, are nearly always based on a formal description of the syntax of programs. FOL compiler separates the task of analyzing syntax into two distinct parts, lexical analysis and syntax analysis, although this terminology is confusing. The lexical analyzer deals with  small-scale language constructs, such as names and numeric literals. The syntax analyzer deals with the large-scale constructs, such as expressions, statements, and program units. 

There are three reasons why lexical analysis is separated from syntax  analysis:
1.  Simplicity— Techniques for lexical analysis are less complex than those required for syntax analysis, so the  lexical-  analysis process can be simpler if it is separate. Also, removing the  low- level details of lexical analysis from the syntax analyzer makes the syntax analyzer both smaller and less complex.
2.  Efficiency— Although it pays to optimize the lexical analyzer, because lexical analysis requires a significant portion of total compilation time, it is not fruitful to optimize the syntax analyzer. Separation facilitates this selective optimization.
3.  Portability— Because the lexical analyzer reads input program files and often includes buffering of that input, it is somewhat platform dependent. However, the syntax analyzer can be platform independent. It is always good to isolate  machine- dependent parts of any software system.
