(source_file) @local.scope
(block) @local.scope

(fun_decl name: (identifier) @local.definition.function)
(log_decl name: (identifier) @local.definition.function)
(typ_decl name: (identifier) @local.definition.type)
(ali_decl name: (identifier) @local.definition.type)
(param name: (identifier) @local.definition)
(var_decl (typed_binding name: (identifier) @local.definition))

(identifier) @local.reference
