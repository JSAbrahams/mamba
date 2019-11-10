use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::environment::expression_type::ExpressionType;

#[derive(Clone, Debug)]
pub struct State {
    pub in_loop:   bool,
    pub in_handle: bool,
    pub in_class:  Option<ActualTypeName>,
    pub in_match:  Option<ExpressionType>,
    pub handling:  Vec<ActualTypeName>
}

pub enum StateType {
    InLoop,
    InHandle
}

impl State {
    pub fn new() -> State {
        State {
            in_loop:   false,
            in_handle: false,
            in_class:  None,
            in_match:  None,
            handling:  vec![]
        }
    }

    pub fn in_class(&self, class: &ActualTypeName) -> State {
        State { in_class: Some(class.clone()), ..self.clone() }
    }

    pub fn in_match(&self, expr_ty: &ExpressionType) -> State {
        State { in_match: Some(expr_ty.clone()), ..self.clone() }
    }

    pub fn handling(&self, handling: &Vec<ActualTypeName>) -> State {
        State { handling: handling.clone(), in_handle: true, ..self.clone() }
    }

    pub fn as_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: true, ..self.clone() },
            StateType::InHandle => State { in_handle: true, ..self.clone() }
        }
    }

    pub fn as_not_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: false, ..self.clone() },
            StateType::InHandle => State { in_handle: false, ..self.clone() }
        }
    }
}
