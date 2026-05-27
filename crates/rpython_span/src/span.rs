use crate::{BytePos, FileId, SyntaxContext};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    pub file_id: FileId,
    pub start: BytePos,
    pub end: BytePos,
    pub ctxt: SyntaxContext,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SpanData {
    pub file_id: FileId,
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub fn new(file_id: FileId, start: BytePos, end: BytePos) -> Self {
        Self {
            file_id,
            start,
            end,
            ctxt: SyntaxContext::default(),
        }
    }

    pub fn dummy() -> Self {
        Self::new(FileId(0), BytePos(0), BytePos(0))
    }

    pub fn merge(a: Self, b: Self) -> Self {
        debug_assert_eq!(a.file_id, b.file_id);
        Self {
            file_id: a.file_id,
            start: a.start.min(b.start),
            end: a.end.max(b.end),
            ctxt: a.ctxt,
        }
    }
}
