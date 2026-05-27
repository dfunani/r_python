use rpython_ast::Path;
use rpython_span::Span;
use smol_str::SmolStr;

/// Record of a resolved or attempted import.
#[derive(Clone, Debug)]
pub struct ImportRecord {
    pub path: SmolStr,
    pub alias: SmolStr,
    pub span: Span,
    pub resolved: bool,
}

impl ImportRecord {
    pub fn from_path(path: &Path, alias: Option<SmolStr>, span: Span, resolved: bool) -> Self {
        let path_str: SmolStr = path
            .segments
            .iter()
            .map(|s| s.ident.as_str())
            .collect::<Vec<_>>()
            .join(".")
            .into();
        let alias = alias.unwrap_or_else(|| path_str.clone());
        Self {
            path: path_str,
            alias,
            span,
            resolved,
        }
    }
}
