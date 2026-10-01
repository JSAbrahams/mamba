use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder, Opcode, Value, ValueDef};
use cranelift_frontend::Variable;
use cranelift_module::{FuncId, Module};

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::common::position::Position;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Arith {
    Add,
    Sub,
    Mul,
}

/// `Int` values, as the words `backend::cranelift::runtime` describes.
///
/// A heap integer is reference counted.
/// A variable owns one reference to its value, and so does a temporary until something consumes it.
/// A function owns the arguments it is passed, and its caller owns what it returns.
impl FnLower<'_> {
    /// Call `callee` on `value` if it is a heap integer.
    fn if_heap(&mut self, value: Value, callee: FuncId) {
        let dfg = &self.builder.func.dfg;
        if dfg.value_type(value) != types::I64 {
            return;
        }
        if let ValueDef::Result(inst, _) = dfg.value_def(value) {
            if dfg.insts[inst].opcode() == Opcode::Iconst {
                return;
            }
        }

        let tag = self.builder.ins().band_imm(value, 1);
        let heap = self.builder.create_block();
        let done = self.builder.create_block();
        self.builder.ins().brif(tag, heap, &[], done, &[]);
        self.builder.switch_to_block(heap);
        let callee = self.module.declare_func_in_func(callee, self.builder.func);
        self.builder.ins().call(callee, &[value]);
        self.builder.ins().jump(done, &[]);
        self.builder.switch_to_block(done);
    }

    /// Take ownership of `value`.
    /// A temporary is claimed as is, and a value read from a variable gains a reference.
    pub(super) fn own(&mut self, value: Value) {
        if !self.temps.remove(&value) {
            self.if_heap(value, self.runtime.retain);
        }
    }

    /// Release `value` if it is a temporary that nothing claimed.
    pub(super) fn drop_temp(&mut self, value: Value) {
        if self.temps.remove(&value) {
            self.if_heap(value, self.runtime.release);
        }
    }

    /// Bind `value` to a new variable, which owns it.
    pub(super) fn bind(&mut self, value: Value) -> Variable {
        self.own(value);
        let ty = self.builder.func.dfg.value_type(value);
        let var = self.new_var(ty);
        self.builder.def_var(var, value);
        if ty == types::I64 {
            self.live.push(var);
        }
        var
    }

    /// Replace what `var` owns with `value`.
    pub(super) fn assign(&mut self, var: Variable, value: Value) {
        self.own(value);
        let old = self.builder.use_var(var);
        self.if_heap(old, self.runtime.release);
        self.builder.def_var(var, value);
    }

    /// Release what every variable bound since `live` had `from` entries owns.
    pub(super) fn release_from(&mut self, from: usize) {
        for i in (from..self.live.len()).rev() {
            let value = self.builder.use_var(self.live[i]);
            self.if_heap(value, self.runtime.release);
        }
    }

    /// Return `values` to the caller, which owns them, after releasing every live variable.
    pub(super) fn ret(&mut self, values: &[Value]) {
        for value in values {
            self.own(*value);
        }
        self.release_from(0);
        self.builder.ins().return_(values);
    }

    /// Eighteen decimal digits always fit a small integer, so a longer literal is built up from such pieces.
    pub(super) fn int_literal(&mut self, lit: &str, pos: Position) -> BackendResult<Value> {
        let mut value = None;
        for piece in lit.as_bytes().chunks(18) {
            let digits: i64 = std::str::from_utf8(piece)
                .ok()
                .and_then(|piece| piece.parse().ok())
                .ok_or_else(|| BackendErr::new(pos, &format!("Invalid int literal '{lit}'")))?;
            let digits = self.builder.ins().iconst(types::I64, digits << 1);
            value = Some(match value {
                None => digits,
                Some(value) => {
                    let scale = 10_i64.pow(piece.len() as u32) << 1;
                    let scale = self.builder.ins().iconst(types::I64, scale);
                    let value = self.int_arith(Arith::Mul, value, scale);
                    self.int_arith(Arith::Add, value, digits)
                }
            });
        }
        value.ok_or_else(|| BackendErr::new(pos, &format!("Invalid int literal '{lit}'")))
    }

    /// Two small integers are combined inline.
    /// The runtime takes over when either is a heap integer, or when the result overflows a small integer.
    pub(super) fn int_arith(&mut self, op: Arith, l: Value, r: Value) -> Value {
        let either = self.builder.ins().bor(l, r);
        let heap = self.builder.ins().band_imm(either, 1);
        let (fast, overflow) = match op {
            Arith::Add => {
                let sum = self.builder.ins().iadd(l, r);
                let l_flipped = self.builder.ins().bxor(l, sum);
                let r_flipped = self.builder.ins().bxor(r, sum);
                let overflow = self.builder.ins().band(l_flipped, r_flipped);
                (sum, self.builder.ins().ushr_imm(overflow, 63))
            }
            Arith::Sub => {
                let difference = self.builder.ins().isub(l, r);
                let signs_differ = self.builder.ins().bxor(l, r);
                let l_flipped = self.builder.ins().bxor(l, difference);
                let overflow = self.builder.ins().band(signs_differ, l_flipped);
                (difference, self.builder.ins().ushr_imm(overflow, 63))
            }
            Arith::Mul => {
                let untagged = self.builder.ins().sshr_imm(l, 1);
                let product = self.builder.ins().imul(untagged, r);
                let high = self.builder.ins().smulhi(untagged, r);
                let sign = self.builder.ins().sshr_imm(product, 63);
                (product, self.builder.ins().bxor(high, sign))
            }
        };
        let slow = self.builder.ins().bor(heap, overflow);

        let runtime = self.builder.create_block();
        let done = self.builder.create_block();
        self.builder.append_block_param(done, types::I64);
        self.builder.ins().brif(slow, runtime, &[], done, &[fast]);
        self.builder.switch_to_block(runtime);
        let call = if op == Arith::Mul {
            let callee = self
                .module
                .declare_func_in_func(self.runtime.mul, self.builder.func);
            self.builder.ins().call(callee, &[l, r])
        } else {
            let callee = self
                .module
                .declare_func_in_func(self.runtime.add_sub, self.builder.func);
            let subtract = self
                .builder
                .ins()
                .iconst(types::I64, i64::from(op == Arith::Sub));
            self.builder.ins().call(callee, &[l, r, subtract])
        };
        let result = self.builder.inst_results(call)[0];
        self.builder.ins().jump(done, &[result]);
        self.builder.switch_to_block(done);

        let result = self.builder.block_params(done)[0];
        self.drop_temp(l);
        self.drop_temp(r);
        self.temps.insert(result);
        result
    }

    /// Two small integers compare as their words do, and anything else is compared by the runtime.
    pub(super) fn int_cmp(&mut self, cc: IntCC, l: Value, r: Value) -> Value {
        let either = self.builder.ins().bor(l, r);
        let heap = self.builder.ins().band_imm(either, 1);
        let fast = self.builder.ins().icmp(cc, l, r);

        let runtime = self.builder.create_block();
        let done = self.builder.create_block();
        self.builder.append_block_param(done, types::I8);
        self.builder.ins().brif(heap, runtime, &[], done, &[fast]);
        self.builder.switch_to_block(runtime);
        let callee = self
            .module
            .declare_func_in_func(self.runtime.cmp, self.builder.func);
        let call = self.builder.ins().call(callee, &[l, r]);
        let order = self.builder.inst_results(call)[0];
        let result = self.builder.ins().icmp_imm(cc, order, 0);
        self.builder.ins().jump(done, &[result]);
        self.builder.switch_to_block(done);

        self.drop_temp(l);
        self.drop_temp(r);
        self.builder.block_params(done)[0]
    }

    pub(super) fn int_to_float(&mut self, value: Value) -> Value {
        let heap = self.builder.ins().band_imm(value, 1);
        let untagged = self.builder.ins().sshr_imm(value, 1);
        let fast = self.builder.ins().fcvt_from_sint(types::F64, untagged);

        let runtime = self.builder.create_block();
        let done = self.builder.create_block();
        self.builder.append_block_param(done, types::F64);
        self.builder.ins().brif(heap, runtime, &[], done, &[fast]);
        self.builder.switch_to_block(runtime);
        let callee = self
            .module
            .declare_func_in_func(self.runtime.to_float, self.builder.func);
        let call = self.builder.ins().call(callee, &[value]);
        let result = self.builder.inst_results(call)[0];
        self.builder.ins().jump(done, &[result]);
        self.builder.switch_to_block(done);

        self.drop_temp(value);
        self.builder.block_params(done)[0]
    }
}
