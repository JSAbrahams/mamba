use crate::type_checker::type_result::TypeErr;

#[derive(Clone)]
pub struct State {
    pub in_loop:   bool,
    pub in_handle: bool
}

pub enum StateType {
    InLoop,
    InHandle
}

impl State {
    pub fn new() -> State { State { in_loop: false, in_handle: false } }

    pub fn is(self, state_type: StateType) -> Result<State, TypeErr> {
        match state_type {
            StateType::InLoop => Ok(State { in_loop: true, in_handle: self.in_handle }),
            StateType::InHandle => Ok(State { in_loop: self.in_loop, in_handle: true })
        }
    }
}
