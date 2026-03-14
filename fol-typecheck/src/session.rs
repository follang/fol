use crate::{decls, exprs, TypecheckResult, TypedProgram};

#[derive(Debug, Default)]
pub struct TypecheckSession;

impl TypecheckSession {
    pub fn new() -> Self {
        Self
    }

    pub fn check_resolved_program(
        &mut self,
        resolved: fol_resolver::ResolvedProgram,
    ) -> TypecheckResult<TypedProgram> {
        let mut typed = TypedProgram::from_resolved(resolved);
        decls::lower_declaration_signatures(&mut typed)?;
        exprs::type_program(&mut typed)?;
        Ok(typed)
    }
}
