use crate::TyId;
use rpython_span::Span;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A qualified path (`foo.bar.Baz`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

/// One segment of a path, optionally with generic arguments.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathSegment {
    pub ident: SmolStr,
    pub args: Vec<TyId>,
    pub span: Span,
}

impl Path {
    pub fn single(name: SmolStr, span: Span) -> Self {
        Self {
            segments: vec![PathSegment {
                ident: name,
                args: Vec::new(),
                span,
            }],
            span,
        }
    }

    pub fn is_simple_ident(&self) -> Option<&SmolStr> {
        if self.segments.len() == 1 && self.segments[0].args.is_empty() {
            Some(&self.segments[0].ident)
        } else {
            None
        }
    }
}
