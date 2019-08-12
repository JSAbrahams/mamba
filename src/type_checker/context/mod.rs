use crate::type_checker::context::class::{Class, Interface};
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;

pub mod class;
pub mod context;
pub mod field;
pub mod function;

#[derive(Debug)]
pub struct Context {
    pub interfaces: Vec<Interface>,
    pub classes:    Vec<Class>,
    pub fields:     Vec<Field>,
    pub functions:  Vec<Function>
}
