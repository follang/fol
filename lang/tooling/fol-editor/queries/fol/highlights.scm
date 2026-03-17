[
  "use"
  "var"
  "fun"
  "log"
  "typ"
  "ali"
  "when"
  "loop"
  "return"
  "report"
  "break"
  "nil"
] @keyword

(typ_decl name: (identifier) @type)
(ali_decl name: (identifier) @type)
(fun_decl name: (identifier) @function)
(log_decl name: (identifier) @function)
(param name: (identifier) @variable.parameter)
(var_decl name: (identifier) @variable)
(field_init name: (identifier) @property)
(dot_intrinsic name: (identifier) @function.builtin)
(qualified_path (identifier) @namespace)
(string_literal) @string
(integer_literal) @number
(comment) @comment
(doc_comment) @comment.documentation
