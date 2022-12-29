use std::cmp::{max, min, Ordering};
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Copy)]
/// A position represents a rectangle in the source code.
pub struct Position {
    pub start: CaretPos,
    pub end: CaretPos,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.start == self.end {
            write!(f, "({})", self.start)
        } else if self.start.line == self.end.line {
            write!(f, "({}-{})", self.start, self.end.pos)
        } else {
            write!(f, "({}-{})", self.start, self.end)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
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
    pub fn new(start: CaretPos, end: CaretPos) -> Position {
        Position { start, end }
    }

    /// Get the absolute width of a position, which represents a rectangle in
    /// the source code.
    ///
    /// Width is always 1 or greater.
    pub fn get_width(&self) -> usize {
        max(
            1,
            max(
                self.end.pos as i32 - self.start.pos as i32,
                self.start.pos as i32 - self.end.pos as i32,
            ) as usize,
        )
    }

    #[must_use]
    pub fn offset(&self, offset: &CaretPos) -> Position {
        Position { start: self.start.offset(offset), end: self.end.offset(offset) }
    }

    #[must_use]
    pub fn union(&self, other: Position) -> Position {
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

impl From<CaretPos> for Position {
    fn from(caret_pos: CaretPos) -> Self {
        Position::new(caret_pos, caret_pos)
    }
}

impl Default for CaretPos {
    fn default() -> Self {
        CaretPos { line: 1, pos: 1 }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::common::position::{CaretPos, Position};

    #[test]
    fn position_eq() {
        let pos1 = Position::new(CaretPos::new(3, 8), CaretPos::new(2, 9));
        let pos2 = Position::new(CaretPos::new(3, 8), CaretPos::new(2, 9));
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn position_ne() {
        let pos1 = Position::new(CaretPos::new(3, 8), CaretPos::new(2, 9));
        let pos2 = Position::new(CaretPos::new(3, 5), CaretPos::new(2, 9));
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn position_line_before_other() {
        assert!(CaretPos::new(3, 8) < CaretPos::new(4, 5));
    }

    #[test]
    fn position_same_line_before_other() {
        assert!(CaretPos::new(4, 4) < CaretPos::new(4, 5));
    }

    #[test]
    fn position_same_line_before_other_leq() {
        assert!(CaretPos::new(4, 4) <= CaretPos::new(4, 5));
    }

    #[test]
    fn position_different_line_before_other_leq() {
        assert!(CaretPos::new(3, 4) <= CaretPos::new(4, 4));
    }

    #[test]
    fn position_same_line_after_other_geq() {
        assert!(CaretPos::new(4, 6) >= CaretPos::new(4, 5));
    }

    #[test]
    fn position_different_line_after_other_geq() {
        assert!(CaretPos::new(5, 4) >= CaretPos::new(4, 4));
    }

    #[test]
    fn position_same_line_before_other_eq() {
        let pos1 = CaretPos::new(4, 5);
        let pos2 = CaretPos::new(4, 5);

        assert!(pos1 <= pos2);
        assert!(pos1 >= pos2);
    }

    #[test]
    fn position_line_before_other_le() {
        assert!(CaretPos::new(4, 5) > CaretPos::new(3, 8));
    }

    #[test]
    fn position_same_line_before_other_le() {
        assert!(CaretPos::new(4, 5) > CaretPos::new(4, 4));
    }

    #[test]
    fn partial_ord_caret_pos() {
        assert_eq!(CaretPos::new(4, 5).partial_cmp(&CaretPos::new(4, 5)), Some(Ordering::Equal));
        assert_eq!(CaretPos::new(4, 4).partial_cmp(&CaretPos::new(4, 5)), Some(Ordering::Less));
        assert_eq!(CaretPos::new(4, 6).partial_cmp(&CaretPos::new(4, 5)), Some(Ordering::Greater));
    }
}
