//! Recursive-descent parser for rPython.

mod expr;
mod item;
mod parser;
mod stmt;

use rpython_ast::{Arena, Module};
use rpython_errors::Handler;
use rpython_span::Span;
use rpython_syntax::TokenStream;

pub use parser::Parser;

/// Parse a token stream into a module AST.
pub fn parse_module(tokens: TokenStream, arena: &Arena, handler: &mut Handler) -> Option<Module> {
    let slice = tokens.tokens();
    if slice.is_empty() {
        return None;
    }
    let start = slice[0].span;
    #[allow(invalid_reference_casting)]
    let arena_mut = unsafe { &mut *(arena as *const Arena as *mut Arena) };
    let mut parser = Parser::new(slice, arena_mut, handler);
    parser.skip_stmt_separators();
    let items = parser.parse_module_items()?;
    parser.skip_stmt_separators();
    if !parser.is_at_end() && !matches!(parser.current_kind(), rpython_syntax::TokenKind::Eof) {
        parser.error(
            parser.current().span,
            "unexpected tokens after module items",
        );
    }
    let end = slice.last().map(|t| t.span).unwrap_or(start);
    let span = Span::merge(start, end);
    Some(Module { items, span })
}
