mod error;
mod indent;
mod lexer;
mod token;

pub use error::LexError;
pub use lexer::{Lexer, TokenStream};
pub use token::{IntLiteral, SpannedToken, TokenKind};

use rpython_errors::Handler;
use rpython_span::{FileId, SourceMap};

/// Tokenize a single source file already loaded in `source_map`.
pub fn tokenize(source_map: &SourceMap, file_id: FileId, handler: &mut Handler) -> TokenStream {
    let file = source_map.file(file_id).expect("file not in source map");
    Lexer::from_source(file_id, &file.contents, handler)
}
