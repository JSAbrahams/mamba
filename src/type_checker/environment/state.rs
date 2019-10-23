#[derive(Clone)]
pub struct State {
    pub in_loop:   bool,
    pub in_handle: bool,
    pub nullable:  bool
}

pub enum StateType {
    InLoop,
    InHandle,
    Nullable
}

impl State {
    pub fn new() -> State { State { in_loop: false, in_handle: false, nullable: true } }

    pub fn as_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: true, ..self.clone() },
            StateType::InHandle => State { in_handle: true, ..self.clone() },
            StateType::Nullable => State { nullable: false, ..self.clone() }
        }
    }

    pub fn as_not_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: false, ..self.clone() },
            StateType::InHandle => State { in_handle: false, ..self.clone() },
            StateType::Nullable => State { nullable: false, ..self.clone() }
        }
    }
}
