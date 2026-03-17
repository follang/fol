(source_file) @symbol.scope
(block) @symbol.scope

(fun_decl name: (identifier) @symbol.function)
(log_decl name: (identifier) @symbol.function)
(typ_decl name: (identifier) @symbol.type)
(ali_decl name: (identifier) @symbol.type)
(var_decl (typed_binding name: (identifier) @symbol.variable))
(use_decl name: (identifier) @symbol.namespace)
