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
] @keyword

(typ_decl name: (identifier) @type)
(ali_decl name: (identifier) @type)
(fun_decl declaration: (plain_fun_decl name: (identifier) @function))
(fun_decl declaration: (method_decl name: (identifier) @function.method))
(log_decl declaration: (plain_log_decl name: (identifier) @function))
(log_decl declaration: (method_decl name: (identifier) @function.method))
(param name: (identifier) @variable.parameter)
(var_decl (typed_binding name: (identifier) @variable))
(field_init name: (identifier) @property)
(dot_intrinsic name: (identifier) @function.builtin)
(qualified_path (identifier) @namespace)
(nil_literal) @constant.builtin
(string_literal) @string
(integer_literal) @number
(comment) @comment
(doc_comment) @comment.documentation
