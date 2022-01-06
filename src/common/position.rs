use std::cmp::{max, min};
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
        max(1, max(self.end.pos as i32 - self.start.pos as i32, self.start.pos as i32 - self.end.pos as i32))
    }

    #[must_use]
    pub fn offset(&self, offset: &CaretPos) -> Position {
        Position {
            start: self.start.clone().offset(offset),
            end: self.end.clone().offset(offset),
        }
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
    pub fn new(line: usize, pos: usize) -> CaretPos { CaretPos { line, pos } }

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
    pub fn newline(self) -> CaretPos { CaretPos { line: self.line + 1, pos: 1 } }
}

impl Default for Position {
    fn default() -> Self { Position { start: CaretPos::default(), end: CaretPos::default() } }
}

impl Default for CaretPos {
    fn default() -> Self { CaretPos { line: 1, pos: 1 } }
}
