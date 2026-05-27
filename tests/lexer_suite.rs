use std::fs;
use std::path::Path;

use rpython_driver::load_and_tokenize;

#[test]
fn tokenize_hello_example() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello.rpy");
    let (_map, tokens) = load_and_tokenize(&path).expect("tokenize hello.rpy");
    assert!(tokens.contains("KwDef"));
    assert!(tokens.contains("Ident(main)"));
}

#[test]
fn lexer_fixtures_match_expectations() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lexer");
    for entry in fs::read_dir(&dir).expect("lexer fixtures dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rpy") {
            continue;
        }
        let expect_path = path.with_extension("tokens.expect");
        let expect = fs::read_to_string(&expect_path)
            .unwrap_or_else(|_| panic!("missing {}", expect_path.display()));
        let (_map, got) = load_and_tokenize(&path).expect("tokenize fixture");
        assert_eq!(
            got.trim(),
            expect.trim(),
            "fixture {}",
            path.file_name().unwrap().to_string_lossy()
        );
    }
}
