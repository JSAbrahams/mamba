use crate::type_checker::type_result::TypeErr;

#[derive(Clone)]
pub struct State {
    pub in_loop:   bool,
    pub in_handle: bool,
    pub safe:      bool
}

pub enum StateType {
    InLoop,
    InHandle,
    Unsafe
}

impl State {
    pub fn new() -> State { State { in_loop: false, in_handle: false, safe: true } }

    pub fn is(self, state_type: StateType) -> Result<State, TypeErr> {
        match state_type {
            StateType::InLoop => Ok(State { in_loop: true, ..self }),
            StateType::InHandle => Ok(State { in_handle: true, ..self }),
            StateType::Unsafe => Ok(State { safe: false, ..self })
        }
    }
}
