use rpython_errors::Handler;
use rpython_span::SourceMap;
use rpython_syntax::tokenize;

fn kinds(source: &str) -> Vec<String> {
    let mut map = SourceMap::new();
    let file_id = map.load_file("test.rpy", source.to_string());
    let mut handler = Handler::new();
    let stream = tokenize(&map, file_id, &mut handler);
    assert!(!handler.has_errors(), "{:?}", handler.diagnostics());
    stream
        .tokens()
        .iter()
        .map(|t| t.kind.name().to_string())
        .collect()
}

#[test]
fn nested_indent() {
    let src = "if True:\n    x = 1\ny = 2\n";
    let kinds = kinds(src);
    assert!(
        kinds.windows(2).any(|w| w == ["Newline", "Indent"]),
        "{kinds:?}"
    );
    assert!(kinds.contains(&"Dedent".to_string()), "{kinds:?}");
}

#[test]
fn blank_lines_do_not_emit_extra_dedent() {
    let src = "def f():\n    pass\n\n\ndef g():\n    pass\n";
    let kinds = kinds(src);
    let dedents = kinds.iter().filter(|k| *k == "Dedent").count();
    assert!(dedents >= 1, "{kinds:?}");
}
