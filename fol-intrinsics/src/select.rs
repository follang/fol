use crate::{reserved_intrinsic_for_surface, IntrinsicEntry, IntrinsicSurface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicSelectionErrorKind {
    UnknownName,
    WrongSurface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrinsicSelectionError {
    pub kind: IntrinsicSelectionErrorKind,
    pub surface: IntrinsicSurface,
    pub name: String,
}

pub fn select_intrinsic(
    surface: IntrinsicSurface,
    name: &str,
) -> Result<&'static IntrinsicEntry, IntrinsicSelectionError> {
    if let Some(entry) = reserved_intrinsic_for_surface(surface, name) {
        return Ok(entry);
    }

    if [
        IntrinsicSurface::DotRootCall,
        IntrinsicSurface::KeywordCall,
        IntrinsicSurface::Postfix,
        IntrinsicSurface::OperatorAlias,
    ]
    .into_iter()
    .filter(|candidate| *candidate != surface)
    .any(|candidate| reserved_intrinsic_for_surface(candidate, name).is_some())
    {
        return Err(IntrinsicSelectionError {
            kind: IntrinsicSelectionErrorKind::WrongSurface,
            surface,
            name: name.to_string(),
        });
    }

    Err(IntrinsicSelectionError {
        kind: IntrinsicSelectionErrorKind::UnknownName,
        surface,
        name: name.to_string(),
    })
}
