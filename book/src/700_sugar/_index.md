# Language Sugar

Language sugar, is a visually or logically-appealing "shortcut" provided by the language, which reduces the amount of code that must be written in some common situation. It makes the language "sweeter" for human use: things can be expressed more clearly, more concisely, or in an alternative style that some may prefer.

A construct in a language is called "language sugar" if it can be removed from the language without any effect on what the language can do: functionality and expressive power will remain the same.

{{% notice info %}}

Language sugars don't add functionality to a language, they are plain textual replacements for expressions that could also be written in a more analytic way.

{{% /notice %}}

There are lots of pros and cons to language sugar. The goal generally being to balance the amount of it available in a language so as to maximise readability -- giving enough freedom to allow the author to emphasize what is important, while being restrictive enough that readers will know what to expect. 

Too much language sugar can make the underlying semantics unclear, but too little can obscure what is being expressed. The author of a piece of code chooses from the available syntax in order to emphasize the important aspects of what it does, and push the minor ones aside. Too much freedom in doing this can make code unreadable because there are many special cases in the syntax for expressing peculiar things which the reader must be familiar with. Too little freedom can also result in unreadability because the author has no way to emphasize any particular way of thinking about the code, even though there may be many ways to look at it when someone new to the code starts reading. It can also make it harder to write code, because there are fewer ways to express any given idea, so it can be more difficult to find one which suits you. One can argue that this is a task for comments to perform, but when it comes time to read the code, it is really the code itself that is readable or not.
