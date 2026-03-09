use super::*;

#[test]
fn test_macro_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_macro.fol")
        .expect("Should read macro definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse macro definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::DefDecl {
                    name,
                    params,
                    def_type: FolType::Named { name: def_kind },
                    body,
                    ..
                }
                if name == "$"
                    && def_kind == "mac"
                    && params.len() == 1
                    && params[0].name == "a"
                    && matches!(params[0].param_type, FolType::Any)
                    && body.len() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_alternative.fol")
        .expect("Should read alternative definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::DefDecl {
                    name,
                    params,
                    def_type: FolType::Named { name: def_kind },
                    body,
                    ..
                }
                if name == "+var"
                    && params.is_empty()
                    && def_kind == "alt"
                    && body.len() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_default_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_default.fol")
        .expect("Should read default definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::DefDecl {
                    name,
                    params,
                    def_type: FolType::Named { name: def_kind },
                    body,
                    ..
                }
                if name == "str"
                    && params.is_empty()
                    && def_kind == "def[]"
                    && body.len() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_only_macro_definitions_accept_parameter_headers() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_alt_with_params.fol")
        .expect("Should read invalid parameterized alternative definition file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject parameters on non-macro definitions");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Definition parameters are currently supported only for mac definitions"),
        "Non-macro parameterized defs should report the parameter restriction, got: {}",
        parse_error
    );
}
