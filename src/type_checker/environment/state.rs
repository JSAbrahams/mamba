use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::type_name::actual::ActualTypeName;
use crate::type_checker::type_name::TypeName;

#[derive(Clone, Debug)]
pub struct State {
    pub in_loop:      bool,
    pub in_handle:    bool,
    pub in_function:  bool,
    pub in_match:     Option<ExpressionType>,
    pub ret_ty:       Option<Expected>,
    pub in_class:     Option<TypeName>,
    pub in_class_new: Option<Expect>,
    pub handling:     Vec<ActualTypeName>
}

pub enum StateType {
    InLoop,
    InHandle,
    InFunction
}

impl Default for State {
    fn default() -> Self {
        State {
            in_loop:      false,
            in_handle:    false,
            in_function:  false,
            in_match:     None,
            in_class:     None,
            ret_ty:       None,
            in_class_new: None,
            handling:     vec![]
        }
    }
}

impl State {
    pub fn in_match(&self, expr_ty: &ExpressionType) -> State {
        State { in_match: Some(expr_ty.clone()), ..self.clone() }
    }

    pub fn ret_ty(&self, ret_ty: &Expected) -> State {
        State { ret_ty: Some(ret_ty.clone()), ..self.clone() }
    }

    pub fn handling(&self, handling: &[ActualTypeName]) -> State {
        State { handling: Vec::from(handling), in_handle: true, ..self.clone() }
    }

    pub fn in_class_new(&self, in_class: &Expect) -> State {
        State { in_class_new: Some(in_class.clone()), ..self.clone() }
    }

    pub fn in_class(&self, in_class: &TypeName) -> State {
        State { in_class: Some(in_class.clone()), ..self.clone() }
    }

    pub fn as_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: true, ..self.clone() },
            StateType::InHandle => State { in_handle: true, ..self.clone() },
            StateType::InFunction => State { in_function: true, ..self.clone() }
        }
    }

    pub fn as_not_state(&self, state_type: StateType) -> State {
        match state_type {
            StateType::InLoop => State { in_loop: false, ..self.clone() },
            StateType::InHandle => State { in_handle: false, ..self.clone() },
            StateType::InFunction => State { in_function: false, ..self.clone() }
        }
    }
}
