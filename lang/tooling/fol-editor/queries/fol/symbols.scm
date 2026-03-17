(source_file) @symbol.scope
(block) @symbol.scope

(fun_decl declaration: (plain_fun_decl name: (identifier) @symbol.function))
(fun_decl declaration: (method_decl name: (identifier) @symbol.method))
(log_decl declaration: (plain_log_decl name: (identifier) @symbol.function))
(log_decl declaration: (method_decl name: (identifier) @symbol.method))
(typ_decl name: (identifier) @symbol.type)
(ali_decl name: (identifier) @symbol.type)
(var_decl (typed_binding name: (identifier) @symbol.variable))
(use_decl name: (identifier) @symbol.namespace)
