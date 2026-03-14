use crate::{decls, exprs, TypecheckError, TypecheckErrorKind, TypecheckResult, TypedPackage, TypedProgram, TypedWorkspace};
use std::collections::{BTreeMap, BTreeSet};

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

    pub fn check_resolved_workspace(
        &mut self,
        resolved: fol_resolver::ResolvedWorkspace,
    ) -> TypecheckResult<TypedWorkspace> {
        let mut typed_packages = BTreeMap::new();
        let mut in_progress = BTreeSet::new();
        let identities = resolved
            .packages()
            .map(|package| package.identity.clone())
            .collect::<Vec<_>>();
        let mut errors = Vec::new();

        for identity in identities {
            if let Err(mut package_errors) = self.check_workspace_package(
                &resolved,
                &identity,
                &mut typed_packages,
                &mut in_progress,
            ) {
                errors.append(&mut package_errors);
            }
        }

        if errors.is_empty() {
            Ok(TypedWorkspace::new(
                resolved.entry_identity().clone(),
                typed_packages,
            ))
        } else {
            Err(errors)
        }
    }

    fn check_workspace_package(
        &mut self,
        workspace: &fol_resolver::ResolvedWorkspace,
        identity: &fol_resolver::PackageIdentity,
        typed_packages: &mut BTreeMap<fol_resolver::PackageIdentity, TypedPackage>,
        in_progress: &mut BTreeSet<fol_resolver::PackageIdentity>,
    ) -> TypecheckResult<()> {
        if typed_packages.contains_key(identity) {
            return Ok(());
        }

        if !in_progress.insert(identity.clone()) {
            return Err(vec![TypecheckError::new(
                TypecheckErrorKind::Internal,
                format!(
                    "typecheck workspace entered a package cycle at '{}'",
                    identity.canonical_root
                ),
            )]);
        }

        let package = workspace.package(identity).ok_or_else(|| {
            vec![TypecheckError::new(
                TypecheckErrorKind::Internal,
                format!(
                    "resolved workspace lost package '{}'",
                    identity.canonical_root
                ),
            )]
        })?;

        let dependency_identities = package
            .program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.mounted_from.as_ref())
            .map(|provenance| provenance.package_identity.clone())
            .filter(|dependency| dependency != identity)
            .collect::<BTreeSet<_>>();

        let mut errors = Vec::new();
        for dependency in dependency_identities {
            if workspace.package(&dependency).is_none() {
                continue;
            }
            if let Err(mut dependency_errors) =
                self.check_workspace_package(workspace, &dependency, typed_packages, in_progress)
            {
                errors.append(&mut dependency_errors);
            }
        }

        if errors.is_empty() {
            let mut typed = TypedProgram::from_resolved(package.program.clone());
            if let Err(mut package_errors) = decls::lower_declaration_signatures(&mut typed) {
                errors.append(&mut package_errors);
            } else if let Err(mut package_errors) = exprs::type_program(&mut typed) {
                errors.append(&mut package_errors);
            } else {
                typed_packages.insert(identity.clone(), TypedPackage::new(identity.clone(), typed));
            }
        }

        in_progress.remove(identity);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
