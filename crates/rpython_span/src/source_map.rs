use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::Span;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct FileId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct BytePos(pub u32);

impl BytePos {
    pub fn advance(self, n: u32) -> Self {
        Self(self.0.saturating_add(n))
    }
}

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub name: PathBuf,
    pub contents: String,
    pub line_starts: Vec<BytePos>,
}

#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    files: IndexMap<FileId, SourceFile>,
    next_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>, contents: String) -> FileId {
        let path = path.as_ref().to_path_buf();
        let line_starts = compute_line_starts(&contents);
        let id = FileId(self.next_id);
        self.next_id += 1;
        self.files.insert(
            id,
            SourceFile {
                name: path,
                contents,
                line_starts,
            },
        );
        id
    }

    pub fn file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(&id)
    }

    pub fn line_col(&self, span: Span) -> LineCol {
        let file = self
            .files
            .get(&span.file_id)
            .expect("unknown file id in span");
        line_col_at(&file.line_starts, span.start)
    }
}

fn compute_line_starts(contents: &str) -> Vec<BytePos> {
    let mut starts = vec![BytePos(0)];
    for (i, b) in contents.bytes().enumerate() {
        if b == b'\n' {
            starts.push(BytePos((i + 1) as u32));
        }
    }
    starts
}

fn line_col_at(line_starts: &[BytePos], pos: BytePos) -> LineCol {
    let line = line_starts
        .partition_point(|start| *start <= pos)
        .saturating_sub(1);
    let col = pos
        .0
        .saturating_sub(line_starts.get(line).map(|p| p.0).unwrap_or(0));
    LineCol {
        line: line + 1,
        col: col as usize + 1,
    }
}
