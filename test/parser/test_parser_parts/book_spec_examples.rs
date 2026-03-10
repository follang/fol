use super::*;

#[test]
fn test_book_generic_object_type_example() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_book_generic_object_type.fol")
            .expect("Should read generic object type example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book generic object type example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    generics,
                    contracts,
                    type_def: TypeDefinition::Record { fields, members, .. },
                    ..
                }
                if name == "container"
                    && generics.len() == 2
                    && contracts.is_empty()
                    && matches!(fields.get("items"), Some(FolType::Sequence { .. }))
                    && members.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "getsize"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_book_extended_aliases_example() {
    let mut file_stream = FileStream::from_file("test/parser/simple_book_extended_aliases.fol")
        .expect("Should read extended aliases example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book extended aliases example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    options,
                    type_def: TypeDefinition::Alias { target: FolType::Int { .. } },
                    ..
                } if name == "int" && options.contains(&fol_parser::ast::TypeOption::Extension)
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    options,
                    type_def: TypeDefinition::Alias { target: FolType::Named { name: target } },
                    ..
                } if name == "str"
                    && target == "str"
                    && options.contains(&fol_parser::ast::TypeOption::Extension)
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_book_protocol_standard_example() {
    let mut file_stream = FileStream::from_file("test/parser/simple_book_standard_protocol.fol")
        .expect("Should read protocol standard example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book protocol standard example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind, body, .. }
                if name == "geometry"
                    && matches!(kind, fol_parser::ast::StandardKind::Protocol)
                    && body.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "area"))
                    && body.iter().any(|member| matches!(member, AstNode::ProDecl { name, .. } if name == "draw"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_book_blueprint_standard_example() {
    let mut file_stream = FileStream::from_file("test/parser/simple_book_standard_blueprint.fol")
        .expect("Should read blueprint standard example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book blueprint standard example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind, body, .. }
                if name == "geometry"
                    && matches!(kind, fol_parser::ast::StandardKind::Blueprint)
                    && body.iter().any(|member| matches!(member, AstNode::VarDecl { name, .. } if name == "width"))
                    && body.iter().any(|member| matches!(member, AstNode::VarDecl { name, options, .. } if name == "name" && options.contains(&fol_parser::ast::VarOption::Immutable)))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_book_extended_standard_example() {
    let mut file_stream = FileStream::from_file("test/parser/simple_book_standard_extended.fol")
        .expect("Should read extended standard example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book extended standard example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind, body, .. }
                if name == "geometry"
                    && matches!(kind, fol_parser::ast::StandardKind::Extended)
                    && body.iter().any(|member| matches!(member, AstNode::VarDecl { name, .. } if name == "width"))
                    && body.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "area"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
