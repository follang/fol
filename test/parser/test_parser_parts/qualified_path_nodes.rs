use super::*;

#[test]
fn test_qualified_path_helpers_preserve_segment_structure() {
    let path = QualifiedPath::new(vec![
        "io".to_string(),
        "console".to_string(),
        "writer".to_string(),
    ]);

    assert!(path.is_qualified());
    assert_eq!(
        path.segments,
        vec!["io".to_string(), "console".to_string(), "writer".to_string()]
    );
    assert_eq!(path.joined(), "io::console::writer");
}

#[test]
fn test_qualified_path_from_joined_keeps_single_and_multi_segment_shapes() {
    let single = QualifiedPath::from_joined("value");
    assert!(!single.is_qualified());
    assert_eq!(single.segments, vec!["value".to_string()]);
    assert_eq!(single.joined(), "value");

    let multi = QualifiedPath::from_joined("pkg::cache::entry");
    assert!(multi.is_qualified());
    assert_eq!(
        multi.segments,
        vec!["pkg".to_string(), "cache".to_string(), "entry".to_string()]
    );
    assert_eq!(multi.joined(), "pkg::cache::entry");
}
