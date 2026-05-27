use rpython_ast::{Arena, ExprKind, StmtKind};
use rpython_errors::Handler;
use rpython_parse::parse_module;
use rpython_span::SourceMap;
use rpython_syntax::tokenize;

#[test]
fn return_with_value_is_not_split_into_two_stmts() {
    let src = "def main() -> int:\n    return 0\n";
    let mut map = SourceMap::new();
    let file = map.load_file(std::path::Path::new("test.rpy"), src.to_string());
    let mut handler = Handler::new();
    let stream = tokenize(&map, file, &mut handler);
    assert!(!handler.has_errors());

    let arena = Arena::new();
    let module = parse_module(stream, &arena, &mut handler).expect("parse");
    assert!(!handler.has_errors());

    let item = arena.item(module.items[0]);
    let rpython_ast::ItemKind::Function { body, .. } = &item.kind else {
        panic!("expected function");
    };
    assert_eq!(body.len(), 1);
    let stmt = arena.stmt(body[0]);
    let StmtKind::Return(Some(expr)) = &stmt.kind else {
        panic!("expected return with value, got {:?}", stmt.kind);
    };
    let expr = arena.expr(*expr);
    assert!(matches!(
        expr.kind,
        ExprKind::Literal(rpython_ast::Literal::Int(0))
    ));
}
