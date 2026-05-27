use rpython_ast::{Arena, ItemKind, StmtKind};
use rpython_errors::Handler;
use rpython_parse::parse_module;
use rpython_span::SourceMap;
use rpython_syntax::tokenize;

#[test]
fn parses_annotated_local_assign() {
    let src = "def main() -> int:\n    a: str = \"hello\"\n    return 0\n";
    let mut map = SourceMap::new();
    let file = map.load_file(std::path::Path::new("t.rpy"), src.to_string());
    let mut handler = Handler::new();
    let stream = tokenize(&map, file, &mut handler);
    assert!(!handler.has_errors());

    let arena = Arena::new();
    let module = parse_module(stream, &arena, &mut handler).expect("parse");
    assert!(!handler.has_errors());

    let item = arena.item(module.items[0]);
    let ItemKind::Function { body, .. } = &item.kind else {
        panic!("expected function");
    };
    let stmt = arena.stmt(body[0]);
    assert!(matches!(stmt.kind, StmtKind::AnnAssign { .. }));
}
