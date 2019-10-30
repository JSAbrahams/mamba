use crate::type_checker::context::type_name::actual::ActualTypeName;

#[derive(Clone)]
pub struct State {
    pub in_loop:   bool,
    pub in_handle: bool,
    pub nullable:  bool,
    pub in_class:  Option<ActualTypeName>
}

pub enum StateType {
    InLoop,
    InHandle,
    Nullable
}

impl State {
    pub fn new() -> State {
        State { in_loop: false, in_handle: false, nullable: true, in_class: None }
    }

    pub fn in_class(&self, class: &ActualTypeName) -> State {
        State { in_class: Some(class.clone()), ..self.clone() }
    }

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
