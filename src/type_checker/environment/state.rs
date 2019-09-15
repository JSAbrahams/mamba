use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone)]
pub struct State {
    pub in_loop:   bool,
    pub unhandled: Vec<Type>
}

pub enum StateType {
    InLoop
}

impl State {
    pub fn new() -> State { State { in_loop: false, unhandled: vec![] } }

    pub fn unhandled(self, raises: &Vec<TypeName>) -> State {
        State { in_loop: self.in_loop, unhandled: raises.clone() }
    }

    pub fn is(self, state_type: StateType) -> Result<State, TypeErr> {
        match state_type {
            InLoop => Ok(State { in_loop: true, unhandled: self.unhandled })
        }
    }
}
