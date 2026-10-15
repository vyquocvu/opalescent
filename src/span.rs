#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }

    pub fn combine(self, other: Span) -> Self {
        let (start, line, col) = if self.start <= other.start {
            (self.start, self.line, self.col)
        } else {
            (other.start, other.line, other.col)
        };
        let end = self.end.max(other.end);
        Self { start, end, line, col }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0, line: 1, col: 1 }
    }
}
