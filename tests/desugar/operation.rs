use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

macro_rules! verify_op {
    ($op:ident) => {{
        let add_op = to_pos!(ASTNode::$op);
        let core = desugar(&add_op);
        assert_eq!(core, Core::$op);
    }};
}

macro_rules! verify {
    ($ast:ident) => {{
        let left = ASTNode::Id { lit: String::from("left") };
        let right = ASTNode::Id { lit: String::from("right") };
        let add_node = to_pos!(ASTNode::$ast { left: to_pos!(left), right: to_pos!(right) });

        let (left, right) = match desugar(&add_node) {
            Core::$ast { left, right } => (left, right),
            other => panic!("Expected mul but was {:?}", other)
        };

        assert_eq!(*left, Core::Id { lit: String::from("left") });
        assert_eq!(*right, Core::Id { lit: String::from("right") });
    }};
}

macro_rules! verify_unary {
    ($ast:ident) => {{
        let expr = to_pos!(ASTNode::Id { lit: String::from("expression") });
        let add_node = to_pos!(ASTNode::$ast { expr });

        let expr_des = match desugar(&add_node) {
            Core::$ast { expr } => expr,
            other => panic!("Expected mul but was {:?}", other)
        };

        assert_eq!(*expr_des, Core::Id { lit: String::from("expression") });
    }};
}

#[test]
fn add_verify() {
    verify!(Add);
}

#[test]
fn sub_verify() {
    verify!(Sub);
}

#[test]
fn mul_verify() {
    verify!(Mul);
}

#[test]
fn div_verify() {
    verify!(Div);
}

#[test]
fn mod_verify() {
    verify!(Mod);
}

#[test]
fn pow_verify() {
    verify!(Pow);
}

#[test]
fn add_unary_verify() {
    verify_unary!(AddU);
}

#[test]
fn sub_unary_verify() {
    verify_unary!(SubU);
}

#[test]
fn sqrt_verify() {
    verify_unary!(Sqrt);
}

#[test]
fn le_verify() {
    verify!(Le);
}

#[test]
fn leq_verify() {
    verify!(Leq);
}

#[test]
fn ge_verify() {
    verify!(Ge);
}

#[test]
fn geq_verify() {
    verify!(Geq);
}

#[test]
fn neq_verify() {
    verify!(Neq);
}

#[test]
fn is_verify() {
    verify!(Is);
}

#[test]
fn not_verify() {
    verify_unary!(Not);
}

#[test]
fn and_verify() {
    verify!(And);
}

#[test]
fn or_verify() {
    verify!(Or);
}

#[test]
fn add_op_verify() {
    verify_op!(AddOp);
}

#[test]
fn sub_op_verify() {
    verify_op!(SubOp);
}

#[test]
#[ignore]
fn sqrt_op_verify() {
    unimplemented!();
}

#[test]
fn mul_op_verify() {
    verify_op!(MulOp);
}

#[test]
fn div_op_verify() {
    verify_op!(DivOp);
}

#[test]
fn pow_op_verify() {
    verify_op!(PowOp);
}

#[test]
fn mod_op_verify() {
    verify_op!(ModOp);
}

#[test]
fn eq_op_verify() {
    verify_op!(EqOp);
}

#[test]
fn le_op_verify() {
    verify_op!(LeOp);
}

#[test]
fn ge_op_verify() {
    verify_op!(GeOp);
}
