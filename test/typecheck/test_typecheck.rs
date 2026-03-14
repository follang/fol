use fol_typecheck::Typechecker;

#[test]
fn typechecker_foundation_smoke_constructs_public_api() {
    let _ = Typechecker::new();
}
