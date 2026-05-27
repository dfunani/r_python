/// Expand tabs to spaces (tab width 8) and return logical column.
pub fn column_width(line: &str) -> usize {
    let mut col = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => col += 1,
            '\t' => col = (col / 8 + 1) * 8,
            _ => break,
        }
    }
    col
}

#[derive(Debug, Default)]
pub struct IndentStack {
    levels: Vec<usize>,
}

impl IndentStack {
    pub fn new() -> Self {
        Self { levels: vec![0] }
    }

    /// Compare new indent level; returns (num_indents, num_dedents).
    pub fn transition(&mut self, new_level: usize) -> Result<(usize, usize), IndentError> {
        let current = *self.levels.last().unwrap_or(&0);
        if new_level > current {
            self.levels.push(new_level);
            Ok((1, 0))
        } else if new_level < current {
            let mut dedents = 0usize;
            while self.levels.last().copied().unwrap_or(0) > new_level {
                self.levels.pop();
                dedents += 1;
            }
            if *self.levels.last().unwrap_or(&0) != new_level {
                return Err(IndentError::Inconsistent);
            }
            Ok((0, dedents))
        } else {
            Ok((0, 0))
        }
    }

    pub fn dedent_to_zero(&mut self) -> usize {
        let mut count = 0;
        while self.levels.len() > 1 {
            self.levels.pop();
            count += 1;
        }
        count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndentError {
    Inconsistent,
}
