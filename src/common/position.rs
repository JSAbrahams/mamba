#[derive(Clone, Debug)]
pub struct Position {
    pub start: EndPoint,
    pub end:   EndPoint
}

#[derive(Clone, Debug)]
pub struct EndPoint {
    pub line: i32,
    pub pos:  i32
}

impl Position {
    pub fn get_width(&self) -> i32 { self.end.pos - self.start.pos }
}

impl EndPoint {
    pub fn offset_line(self, offset: i32) -> EndPoint {
        EndPoint { line: self.line + offset, pos: self.pos }
    }

    pub fn offset_pos(self, offset: i32) -> EndPoint {
        EndPoint { line: self.line, pos: self.pos + offset }
    }
}
