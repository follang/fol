use super::*;

#[test]
fn test_type_declarations_accept_semicolon_option_separators() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_options_semicolon.fol")
        .expect("Should read semicolon type-option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated type options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        options,
                        type_def: TypeDefinition::Alias { .. },
                        ..
                    }
                    if name == "PublicText"
                        && options == &vec![fol_parser::ast::TypeOption::Export]
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        options,
                        type_def: TypeDefinition::Record { .. },
                        ..
                    }
                    if name == "Accessors"
                        && options
                            == &vec![
                                fol_parser::ast::TypeOption::Set,
                                fol_parser::ast::TypeOption::Get
                            ]
                )
            }));
        }
        _ => panic!("Should return Program node"),
    }
}
