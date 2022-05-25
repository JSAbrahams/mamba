use std::cmp::{max, min, Ordering};
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
/// A position represents a rectangle in the source code.
pub struct Position {
    pub start: CaretPos,
    pub end: CaretPos,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.start == self.end {
            write!(f, "({})", self.start)
        } else {
            write!(f, "({}-{})", self.start, self.end)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An endpoint represents either the top left or bottom right points of a
/// [Position] rectangle.
///
/// Line's and position's are 1-indexed.
pub struct CaretPos {
    pub line: usize,
    pub pos: usize,
}

impl PartialOrd for CaretPos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.line == other.line && self.pos == other.pos {
            Some(Ordering::Equal)
        } else if self.line < other.line || (self.line == other.line && self.pos < other.pos) {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }

    fn lt(&self, other: &Self) -> bool {
        self.line < other.line || (self.line == other.line && self.pos < other.pos)
    }

    fn le(&self, other: &Self) -> bool {
        self.line < other.line || (self.line == other.line && self.pos <= other.pos)
    }

    fn gt(&self, other: &Self) -> bool {
        self.line > other.line || (self.line == other.line && self.pos > other.pos)
    }

    fn ge(&self, other: &Self) -> bool {
        self.line > other.line || (self.line == other.line && self.pos >= other.pos)
    }
}

impl Display for CaretPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:{}", self.line, self.pos)
    }
}

impl Position {
    pub fn new(start: &CaretPos, end: &CaretPos) -> Position {
        Position { start: start.clone(), end: end.clone() }
    }

    /// Get the absolute width of a position, which represents a rectangle in
    /// the source code.
    ///
    /// Width is always 1 or greater.
    pub fn get_width(&self) -> i32 {
        max(
            1,
            max(
                self.end.pos as i32 - self.start.pos as i32,
                self.start.pos as i32 - self.end.pos as i32,
            ),
        )
    }

    #[must_use]
    pub fn offset(&self, offset: &CaretPos) -> Position {
        Position { start: self.start.clone().offset(offset), end: self.end.clone().offset(offset) }
    }

    #[must_use]
    pub fn union(&self, other: &Position) -> Position {
        Position {
            start: CaretPos {
                line: min(self.start.line, other.start.line),
                pos: min(self.start.pos, other.start.pos),
            },
            end: CaretPos {
                line: max(self.end.line, other.end.line),
                pos: max(self.end.pos, other.end.pos),
            },
        }
    }
}

impl CaretPos {
    /// Create new endpoint with given line and position.
    pub fn new(line: usize, pos: usize) -> CaretPos {
        CaretPos { line, pos }
    }

    #[must_use]
    pub fn offset(self, offset: &CaretPos) -> CaretPos {
        CaretPos { line: self.line + offset.line - 1, pos: self.pos + offset.pos - 1 }
    }

    /// Create new [EndPoint] which is offset in the vertical direction by the
    /// given amount.
    #[must_use]
    pub fn offset_line(self, offset: usize) -> CaretPos {
        CaretPos { line: (self.line as i32 + offset as i32) as usize, pos: self.pos }
    }

    /// Create new [EndPoint] which is offset in the horizontal direction by the
    /// given amount.
    #[must_use]
    pub fn offset_pos(self, offset: usize) -> CaretPos {
        CaretPos { line: self.line, pos: self.pos + offset }
    }

    #[must_use]
    pub fn newline(self) -> CaretPos {
        CaretPos { line: self.line + 1, pos: 1 }
    }
}

impl From<&CaretPos> for Position {
    fn from(caret_pos: &CaretPos) -> Self {
        Position::new(&caret_pos.clone(), &caret_pos.clone())
    }
}

impl Default for CaretPos {
    fn default() -> Self {
        CaretPos { line: 1, pos: 1 }
    }
}
