use crate::core::construct::Core;

pub type DesugarResult<T = Core> = std::result::Result<T, UnimplementedErr>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct UnimplementedErr {
    pub line: i32,
    pub pos:  i32,
    pub msg:  String
}

impl UnimplementedErr {
    pub fn new(msg: &str) -> UnimplementedErr {
        UnimplementedErr {
            line: 0,
            pos:  0,
            msg:  format!("The {} construct has not yet been implemented as of v{}", msg, VERSION)
        }
    }
}
