use std::cmp::max;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// A position represents a rectangle in the source code.
pub struct Position {
    pub start: EndPoint,
    pub end:   EndPoint
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An endpoint represents either the top left or bottom right points of a
/// [Position] rectangle.
///
/// Line's and position's are 1-indexed.
pub struct EndPoint {
    pub line: i32,
    pub pos:  i32
}

impl Position {
    /// Get the absolute width of a position, which represents a rectangle in
    /// the source code.
    ///
    /// Width is always 1 or greater.
    pub fn get_width(&self) -> i32 {
        max(1, max(self.end.pos - self.start.pos, self.start.pos - self.end.pos))
    }
}

impl EndPoint {
    /// Create new endpoint with given line and position.
    pub fn new(line: i32, pos: i32) -> EndPoint { EndPoint { line, pos } }

    /// Create new [EndPoint] which is offset in the vertical direction by the
    /// given amount.
    pub fn offset_line(self, offset: i32) -> EndPoint {
        EndPoint { line: self.line + offset, pos: self.pos }
    }

    /// Create new [EndPoint] which is offset in the horizontal direction by the
    /// given amount.
    pub fn offset_pos(self, offset: i32) -> EndPoint {
        EndPoint { line: self.line, pos: self.pos + offset }
    }
}
