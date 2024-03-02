# Metaprogramming

Metaprogramming is a programming technique in which computer programs have the ability to treat other programs as their data. It means that a program can be designed to read, generate, analyze or transform other programs, and even modify itself while running. In some cases, this allows programmers to minimize the number of lines of code to express a solution, in turn reducing development time. It also allows programs greater flexibility to efficiently handle new situations without recompilation. 

Macro system (aka template metaprogramming) is a metaprogramming technique in which templates are used by a compiler to generate temporary source code, which is merged by the compiler with the rest of the source code and then compiled. The output of these templates include compile-time constants, data structures, and complete functions. The use of templates can be thought of as compile-time execution.

{{% notice warn %}}

IT IS NOT SUGGESTED TO RELAY HEAVILY ON MACROS BECAUSE THE CODE MIGHT LOOSES THE READABILITY WHEN SOMEONE TRIES TO USE YOUR CODE.

{{% /notice %}}

## Usage

If you have a large application where many of the functions include a lot of boilerplate code, you can create a mini-language that will do the boilerplate code for you and allow you to code only the important parts. Now, if you can, it's best to abstract out the boilerplate portions into a function. But often the boilerplate code isn't so pretty. Maybe there's a list of variables to be declared in every instance, maybe you need to register error handlers, or maybe there are several pieces of the boilerplate that have to have code inserted in certain circumstances. All of these make a simple function call impossible. In such cases, it is often a good idea to create a mini-language that allows you to work with your boilerplate code in an easier fashion. This mini-language will then be converted into your regular source code language before compiling.

Metaprogramming works by circumventing the language. It allows for the alteration of languages through program transformation systems. This procedure gives metaprogramming the freedom to use languages even if the language does not employ any metaprogramming characteristics.

## Types
In FOL there are many templates and metaprogramming elements. 

- Build-In
- Macros
- Alternatives
- Defaults
- Templates


{{% notice tip %}}

WITH BUILD-INS, ALTERNATIVES, MACROS, DEFAULTS AND TEMPLATES, YOU CAN COMPLETELY MAKE A NEW TYPESYSTEM, WITH ITS OWN KEYWORDS, IDENTIFIERS, AND BEHAVIOUR.

{{% /notice %}}


