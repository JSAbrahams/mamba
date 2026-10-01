//! The integer runtime: unbounded `Int` arithmetic, emitted as Cranelift IR into every object.
//!
//! An `Int` is one 64-bit word.
//! A clear low bit marks a small integer, stored shifted left by one.
//! A set low bit marks a pointer to a heap integer, offset by one.
//! A heap integer is a reference count, a sign, a length, and that many base 2^32 limbs, least significant first.
//! A result that fits a small integer is always returned as one, so two small words compare as plain integers.

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::types::{F64, I32, I64};
use cranelift_codegen::ir::{
    AbiParam, Function, InstBuilder, MemFlags, Signature, StackSlotData, StackSlotKind, Type,
    UserFuncName, Value,
};
use cranelift_codegen::Context as ClifContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::common::position::Position;

const REFS: i32 = 0;
const SIGN: i32 = 8;
const LEN: i32 = 16;
const LIMBS: i32 = 24;

/// The runtime functions lowered code calls.
/// Each takes and returns `Int` words, and borrows its arguments.
pub struct Runtime {
    pub retain: FuncId,
    pub release: FuncId,
    /// `a + b`, or `a - b` when the third argument is 1.
    pub add_sub: FuncId,
    pub mul: FuncId,
    /// -1, 0 or 1.
    pub cmp: FuncId,
    pub to_float: FuncId,
    pub print: FuncId,
}

#[derive(Clone, Copy)]
struct Libc {
    calloc: FuncId,
    free: FuncId,
    puts: FuncId,
    exit: FuncId,
}

fn err(e: impl ToString) -> Box<BackendErr> {
    BackendErr::new(Position::invisible(), &e.to_string())
}

fn declare(
    module: &mut ObjectModule,
    name: &str,
    linkage: Linkage,
    params: &[Type],
    returns: &[Type],
) -> BackendResult<(FuncId, Signature)> {
    let mut sig = module.make_signature();
    sig.params
        .extend(params.iter().map(|ty| AbiParam::new(*ty)));
    sig.returns
        .extend(returns.iter().map(|ty| AbiParam::new(*ty)));
    let id = module.declare_function(name, linkage, &sig).map_err(err)?;
    Ok((id, sig))
}

/// Define a function local to this object, with `body` emitting its instructions and its returns.
fn define(
    module: &mut ObjectModule,
    libc: Libc,
    name: &str,
    params: &[Type],
    returns: &[Type],
    body: impl FnOnce(&mut Body, &[Value]),
) -> BackendResult<FuncId> {
    let (id, sig) = declare(module, name, Linkage::Local, params, returns)?;
    let mut ctx = ClifContext::new();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, id.as_u32()), sig);
    let mut fb_ctx = FunctionBuilderContext::new();

    let mut b = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
    let entry = b.create_block();
    b.append_block_params_for_function_params(entry);
    b.switch_to_block(entry);
    let args = b.block_params(entry).to_vec();
    let mut f = Body {
        b,
        module: &mut *module,
        libc,
        vars: 0,
    };
    body(&mut f, &args);
    f.b.seal_all_blocks();
    f.b.finalize();

    module.define_function(id, &mut ctx).map_err(err)?;
    Ok(id)
}

/// Define the runtime in `module`.
/// The names contain a dot, which no Mamba identifier can, so they never clash with a user function.
pub fn define_runtime(module: &mut ObjectModule) -> BackendResult<Runtime> {
    let libc = Libc {
        calloc: declare(module, "calloc", Linkage::Import, &[I64, I64], &[I64])?.0,
        free: declare(module, "free", Linkage::Import, &[I64], &[])?.0,
        puts: declare(module, "puts", Linkage::Import, &[I64], &[I32])?.0,
        exit: declare(module, "exit", Linkage::Import, &[I32], &[])?.0,
    };

    let overflow = module.declare_anonymous_data(false, false).map_err(err)?;
    let mut message = DataDescription::new();
    message.define(Box::from(
        &b"OverflowError: int too large to convert to float\0"[..],
    ));
    module.define_data(overflow, &message).map_err(err)?;

    // A zeroed heap integer of the given length, holding one reference.
    // It has room for at least two limbs, which is what a small integer needs.
    let alloc = define(
        module,
        libc,
        "mamba.int.alloc",
        &[I64],
        &[I64],
        |f, args| {
            let two = f.c(2);
            let roomy = f.b.ins().icmp(IntCC::SignedGreaterThan, args[0], two);
            let capacity = f.b.ins().select(roomy, args[0], two);
            let limbs = f.b.ins().ishl_imm(capacity, 2);
            let size = f.b.ins().iadd_imm(limbs, i64::from(LIMBS));
            let one = f.c(1);
            let p = f.call(f.libc.calloc, &[one, size])[0];
            f.store(one, p, REFS);
            f.store(args[0], p, LEN);
            f.b.ins().return_(&[p]);
        },
    )?;

    // The word for a freshly computed heap integer, which is freed if it fits a small integer.
    let normalize = define(
        module,
        libc,
        "mamba.int.normalize",
        &[I64],
        &[I64],
        |f, args| {
            let p = args[0];
            let len = f.load(p, LEN);
            let len = f.var(len);
            let zero = f.c(0);
            f.while_(
                |f| {
                    let n = f.b.use_var(len);
                    let nonempty = f.b.ins().icmp_imm(IntCC::SignedGreaterThan, n, 0);
                    let last = f.b.ins().iadd_imm(n, -1);
                    let last = f.b.ins().select(nonempty, last, zero);
                    let top = f.limb(p, last);
                    let top_empty = f.b.ins().icmp_imm(IntCC::Equal, top, 0);
                    f.b.ins().band(nonempty, top_empty)
                },
                |f| {
                    let n = f.b.use_var(len);
                    let n = f.b.ins().iadd_imm(n, -1);
                    f.b.def_var(len, n);
                },
            );
            let n = f.b.use_var(len);
            f.store(n, p, LEN);

            let one = f.c(1);
            let low = f.limb(p, zero);
            let high = f.limb(p, one);
            let high = f.b.ins().ishl_imm(high, 32);
            let magnitude = f.b.ins().bor(low, high);
            let sign = f.load(p, SIGN);
            // A small integer holds -2^62 up to 2^62 - 1.
            let limit = f.b.ins().iadd_imm(sign, (1 << 62) - 1);
            let long = f.b.ins().icmp_imm(IntCC::SignedGreaterThan, n, 2);
            let wide = f.b.ins().icmp(IntCC::UnsignedGreaterThan, magnitude, limit);
            let heap = f.b.ins().bor(long, wide);
            let word = f.b.ins().iadd_imm(p, 1);
            f.return_if(heap, &[word]);

            f.call(f.libc.free, &[p]);
            let negated = f.b.ins().ineg(magnitude);
            let value = f.b.ins().select(sign, negated, magnitude);
            let word = f.b.ins().ishl_imm(value, 1);
            f.b.ins().return_(&[word]);
        },
    )?;

    // Compare two magnitudes: -1, 0 or 1.
    let mag_cmp = define(
        module,
        libc,
        "mamba.int.mag_cmp",
        &[I64, I64],
        &[I64],
        |f, args| {
            let (pa, pb) = (args[0], args[1]);
            let la = f.load(pa, LEN);
            let lb = f.load(pb, LEN);
            let less = f.c(-1);
            let greater = f.c(1);
            let shorter = f.b.ins().icmp(IntCC::SignedLessThan, la, lb);
            let by_len = f.b.ins().select(shorter, less, greater);
            let uneven = f.b.ins().icmp(IntCC::NotEqual, la, lb);
            f.return_if(uneven, &[by_len]);

            f.each(la, |f, i| {
                let index = f.b.ins().isub(la, i);
                let index = f.b.ins().iadd_imm(index, -1);
                let x = f.limb(pa, index);
                let y = f.limb(pb, index);
                let lower = f.b.ins().icmp(IntCC::UnsignedLessThan, x, y);
                let by_limb = f.b.ins().select(lower, less, greater);
                let differ = f.b.ins().icmp(IntCC::NotEqual, x, y);
                f.return_if(differ, &[by_limb]);
            });
            let equal = f.c(0);
            f.b.ins().return_(&[equal]);
        },
    )?;

    let retain = define(module, libc, "mamba.int.retain", &[I64], &[], |f, args| {
        let p = f.b.ins().iadd_imm(args[0], -1);
        let refs = f.load(p, REFS);
        let refs = f.b.ins().iadd_imm(refs, 1);
        f.store(refs, p, REFS);
        f.b.ins().return_(&[]);
    })?;

    let release = define(module, libc, "mamba.int.release", &[I64], &[], |f, args| {
        let p = f.b.ins().iadd_imm(args[0], -1);
        let refs = f.load(p, REFS);
        let refs = f.b.ins().iadd_imm(refs, -1);
        f.store(refs, p, REFS);
        f.return_if(refs, &[]);
        f.call(f.libc.free, &[p]);
        f.b.ins().return_(&[]);
    })?;

    let add_sub = define(
        module,
        libc,
        "mamba.int.add_sub",
        &[I64, I64, I64],
        &[I64],
        |f, args| {
            let pa = f.view(args[0]);
            let pb = f.view(args[1]);
            let sa = f.load(pa, SIGN);
            let sb = f.load(pb, SIGN);
            let sb = f.b.ins().bxor(sb, args[2]);

            let order = f.call(mag_cmp, &[pa, pb])[0];
            let a_larger =
                f.b.ins()
                    .icmp_imm(IntCC::SignedGreaterThanOrEqual, order, 0);
            let large = f.b.ins().select(a_larger, pa, pb);
            let small = f.b.ins().select(a_larger, pb, pa);
            let sign = f.b.ins().select(a_larger, sa, sb);
            let large_len = f.load(large, LEN);
            let small_len = f.load(small, LEN);

            let len = f.b.ins().iadd_imm(large_len, 1);
            let r = f.call(alloc, &[len])[0];
            f.store(sign, r, SIGN);

            // Equal signs add the magnitudes.
            // Unequal signs subtract the smaller from the larger, as `large + ~small + 1`.
            let subtract = f.b.ins().icmp(IntCC::NotEqual, sa, sb);
            let zero = f.c(0);
            let ones = f.c(0xffff_ffff);
            let mask = f.b.ins().select(subtract, ones, zero);
            let carry = f.b.ins().uextend(I64, subtract);
            let carry = f.var(carry);
            f.each(large_len, |f, i| {
                let in_small = f.b.ins().icmp(IntCC::SignedLessThan, i, small_len);
                let index = f.b.ins().select(in_small, i, zero);
                let y = f.limb(small, index);
                let y = f.b.ins().select(in_small, y, zero);
                let y = f.b.ins().bxor(y, mask);
                let x = f.limb(large, i);
                let sum = f.b.ins().iadd(x, y);
                let carried = f.b.use_var(carry);
                let sum = f.b.ins().iadd(sum, carried);
                f.set_limb(r, i, sum);
                let sum = f.b.ins().ushr_imm(sum, 32);
                f.b.def_var(carry, sum);
            });
            let carried = f.b.use_var(carry);
            let top = f.b.ins().select(subtract, zero, carried);
            f.set_limb(r, large_len, top);

            let word = f.call(normalize, &[r])[0];
            f.b.ins().return_(&[word]);
        },
    )?;

    let mul = define(
        module,
        libc,
        "mamba.int.mul",
        &[I64, I64],
        &[I64],
        |f, args| {
            let pa = f.view(args[0]);
            let pb = f.view(args[1]);
            let la = f.load(pa, LEN);
            let lb = f.load(pb, LEN);
            let len = f.b.ins().iadd(la, lb);
            let r = f.call(alloc, &[len])[0];
            let sa = f.load(pa, SIGN);
            let sb = f.load(pb, SIGN);
            let sign = f.b.ins().bxor(sa, sb);
            f.store(sign, r, SIGN);

            f.each(la, |f, i| {
                let x = f.limb(pa, i);
                let zero = f.c(0);
                let carry = f.var(zero);
                f.each(lb, |f, j| {
                    let index = f.b.ins().iadd(i, j);
                    let y = f.limb(pb, j);
                    let product = f.b.ins().imul(x, y);
                    let sum = f.limb(r, index);
                    let sum = f.b.ins().iadd(product, sum);
                    let carried = f.b.use_var(carry);
                    let sum = f.b.ins().iadd(sum, carried);
                    f.set_limb(r, index, sum);
                    let sum = f.b.ins().ushr_imm(sum, 32);
                    f.b.def_var(carry, sum);
                });
                let index = f.b.ins().iadd(i, lb);
                let carried = f.b.use_var(carry);
                f.set_limb(r, index, carried);
            });

            let word = f.call(normalize, &[r])[0];
            f.b.ins().return_(&[word]);
        },
    )?;

    let cmp = define(
        module,
        libc,
        "mamba.int.cmp",
        &[I64, I64],
        &[I64],
        |f, args| {
            let pa = f.view(args[0]);
            let pb = f.view(args[1]);
            let sa = f.load(pa, SIGN);
            let sb = f.load(pb, SIGN);
            let order = f.call(mag_cmp, &[pa, pb])[0];
            // Of two negatives, the larger magnitude is the smaller number.
            let reversed = f.b.ins().ineg(order);
            let same_sign = f.b.ins().select(sa, reversed, order);
            let less = f.c(-1);
            let greater = f.c(1);
            let other_sign = f.b.ins().select(sa, less, greater);
            let differ = f.b.ins().icmp(IntCC::NotEqual, sa, sb);
            let result = f.b.ins().select(differ, other_sign, same_sign);
            f.b.ins().return_(&[result]);
        },
    )?;

    // The nearest `Float`, rounding half to even as Python does.
    let to_float = define(
        module,
        libc,
        "mamba.int.to_float",
        &[I64],
        &[F64],
        |f, args| {
            let p = f.view(args[0]);
            let n = f.load(p, LEN);
            let zero = f.c(0);
            let one = f.c(1);
            let low = f.limb(p, zero);
            let high = f.limb(p, one);
            let high = f.b.ins().ishl_imm(high, 32);
            let magnitude = f.b.ins().bor(low, high);
            let exact = f.b.ins().fcvt_from_uint(F64, magnitude);
            let result = f.var(exact);

            let long = f.b.ins().icmp_imm(IntCC::SignedGreaterThan, n, 2);
            f.when(long, |f| {
                // Convert the top 64 bits, with the lowest set if any bit below them is, then scale back up.
                let first = f.b.ins().iadd_imm(n, -1);
                let second = f.b.ins().iadd_imm(n, -2);
                let third = f.b.ins().iadd_imm(n, -3);
                let top = f.limb(p, first);
                let top = f.b.ins().ishl_imm(top, 32);
                let next = f.limb(p, second);
                let top = f.b.ins().bor(top, next);
                let shift = f.b.ins().clz(top);
                let top = f.b.ins().ishl(top, shift);
                let below = f.limb(p, third);
                let below = f.b.ins().ishl(below, shift);
                let fill = f.b.ins().ushr_imm(below, 32);
                let top = f.b.ins().bor(top, fill);

                let rest = f.b.ins().band_imm(below, 0xffff_ffff);
                let rest = f.var(rest);
                f.each(third, |f, i| {
                    let limb = f.limb(p, i);
                    let seen = f.b.use_var(rest);
                    let seen = f.b.ins().bor(seen, limb);
                    f.b.def_var(rest, seen);
                });
                let rest = f.b.use_var(rest);
                let sticky = f.b.ins().icmp_imm(IntCC::NotEqual, rest, 0);
                let sticky = f.b.ins().uextend(I64, sticky);
                let top = f.b.ins().bor(top, sticky);
                let rounded = f.b.ins().fcvt_from_uint(F64, top);

                let exponent = f.b.ins().ishl_imm(second, 5);
                let exponent = f.b.ins().isub(exponent, shift);
                let bits = f.b.ins().iadd_imm(exponent, 1023);
                let bits = f.b.ins().ishl_imm(bits, 52);
                let scale = f.b.ins().bitcast(F64, MemFlags::new(), bits);
                let scaled = f.b.ins().fmul(rounded, scale);
                f.b.def_var(result, scaled);

                let infinity = f.b.ins().f64const(f64::INFINITY);
                let infinite = f.b.ins().fcmp(FloatCC::Equal, scaled, infinity);
                let unscalable = f.b.ins().icmp_imm(IntCC::SignedGreaterThan, exponent, 1023);
                let too_large = f.b.ins().bor(infinite, unscalable);
                f.when(too_large, |f| {
                    let message = f.module.declare_data_in_func(overflow, f.b.func);
                    let message = f.b.ins().global_value(I64, message);
                    f.call(f.libc.puts, &[message]);
                    let status = f.b.ins().iconst(I32, 1);
                    f.call(f.libc.exit, &[status]);
                });
            });

            let value = f.b.use_var(result);
            let negated = f.b.ins().fneg(value);
            let sign = f.load(p, SIGN);
            let value = f.b.ins().select(sign, negated, value);
            f.b.ins().return_(&[value]);
        },
    )?;

    // Print in decimal, followed by a newline.
    let print = define(module, libc, "mamba.int.print", &[I64], &[], |f, args| {
        let p = f.view(args[0]);
        let n = f.load(p, LEN);
        // A scratch copy of the magnitude, divided by 10^9 until nothing is left.
        let scratch = f.call(alloc, &[n])[0];
        f.each(n, |f, i| {
            let limb = f.limb(p, i);
            f.set_limb(scratch, i, limb);
        });
        // Ten digits per limb is an upper bound, and the rest is for a sign and the terminator.
        let size = f.b.ins().imul_imm(n, 10);
        let size = f.b.ins().iadd_imm(size, 3);
        let one = f.c(1);
        let text = f.call(f.libc.calloc, &[one, size])[0];
        let end = f.b.ins().iadd_imm(size, -1);
        let at = f.var(end);

        let left = f.var(n);
        f.while_(
            |f| {
                let left = f.b.use_var(left);
                f.b.ins().icmp_imm(IntCC::SignedGreaterThan, left, 0)
            },
            |f| {
                let len = f.b.use_var(left);
                let zero = f.c(0);
                let chunk = f.var(zero);
                f.each(len, |f, i| {
                    let index = f.b.ins().isub(len, i);
                    let index = f.b.ins().iadd_imm(index, -1);
                    let limb = f.limb(scratch, index);
                    let carried = f.b.use_var(chunk);
                    let carried = f.b.ins().ishl_imm(carried, 32);
                    let dividend = f.b.ins().bor(carried, limb);
                    let quotient = f.b.ins().udiv_imm(dividend, 1_000_000_000);
                    let remainder = f.b.ins().urem_imm(dividend, 1_000_000_000);
                    f.set_limb(scratch, index, quotient);
                    f.b.def_var(chunk, remainder);
                });
                // Dividing empties at most the top limb.
                let top = f.b.ins().iadd_imm(len, -1);
                let top = f.limb(scratch, top);
                let emptied = f.b.ins().icmp_imm(IntCC::Equal, top, 0);
                let emptied = f.b.ins().uextend(I64, emptied);
                let len = f.b.ins().isub(len, emptied);
                f.b.def_var(left, len);

                // Nine digits per chunk, except the leading chunk, which has no leading zeros.
                let written = f.var(zero);
                f.while_(
                    |f| {
                        let count = f.b.use_var(written);
                        let padding = f.b.ins().icmp_imm(IntCC::SignedLessThan, count, 9);
                        let pending = f.b.use_var(chunk);
                        let pending = f.b.ins().icmp_imm(IntCC::NotEqual, pending, 0);
                        f.b.ins().select(len, padding, pending)
                    },
                    |f| {
                        let pending = f.b.use_var(chunk);
                        let digit = f.b.ins().urem_imm(pending, 10);
                        let digit = f.b.ins().iadd_imm(digit, i64::from(b'0'));
                        f.prepend(text, at, digit);
                        let pending = f.b.ins().udiv_imm(pending, 10);
                        f.b.def_var(chunk, pending);
                        let count = f.b.use_var(written);
                        let count = f.b.ins().iadd_imm(count, 1);
                        f.b.def_var(written, count);
                    },
                );
            },
        );

        let start = f.b.use_var(at);
        let blank = f.b.ins().icmp(IntCC::Equal, start, end);
        f.when(blank, |f| {
            let digit = f.c(i64::from(b'0'));
            f.prepend(text, at, digit);
        });
        let sign = f.load(p, SIGN);
        f.when(sign, |f| {
            let minus = f.c(i64::from(b'-'));
            f.prepend(text, at, minus);
        });
        let start = f.b.use_var(at);
        let line = f.b.ins().iadd(text, start);
        f.call(f.libc.puts, &[line]);
        f.call(f.libc.free, &[text]);
        f.call(f.libc.free, &[scratch]);
        f.b.ins().return_(&[]);
    })?;

    Ok(Runtime {
        retain,
        release,
        add_sub,
        mul,
        cmp,
        to_float,
        print,
    })
}

/// A runtime function under construction.
struct Body<'a> {
    b: FunctionBuilder<'a>,
    module: &'a mut ObjectModule,
    libc: Libc,
    vars: u32,
}

impl Body<'_> {
    fn c(&mut self, n: i64) -> Value {
        self.b.ins().iconst(I64, n)
    }

    fn var(&mut self, init: Value) -> Variable {
        let var = Variable::from_u32(self.vars);
        self.vars += 1;
        let ty = self.b.func.dfg.value_type(init);
        self.b.declare_var(var, ty);
        self.b.def_var(var, init);
        var
    }

    fn call(&mut self, id: FuncId, args: &[Value]) -> &[Value] {
        let callee = self.module.declare_func_in_func(id, self.b.func);
        let call = self.b.ins().call(callee, args);
        self.b.inst_results(call)
    }

    fn load(&mut self, p: Value, offset: i32) -> Value {
        self.b.ins().load(I64, MemFlags::new(), p, offset)
    }

    fn store(&mut self, value: Value, p: Value, offset: i32) {
        self.b.ins().store(MemFlags::new(), value, p, offset);
    }

    fn limb(&mut self, p: Value, index: Value) -> Value {
        let offset = self.b.ins().ishl_imm(index, 2);
        let address = self.b.ins().iadd(p, offset);
        self.b.ins().uload32(MemFlags::new(), address, LIMBS)
    }

    fn set_limb(&mut self, p: Value, index: Value, value: Value) {
        let offset = self.b.ins().ishl_imm(index, 2);
        let address = self.b.ins().iadd(p, offset);
        self.b
            .ins()
            .istore32(MemFlags::new(), value, address, LIMBS);
    }

    /// Write `byte` just before the text starting at `text + at`.
    fn prepend(&mut self, text: Value, at: Variable, byte: Value) {
        let index = self.b.use_var(at);
        let index = self.b.ins().iadd_imm(index, -1);
        self.b.def_var(at, index);
        let address = self.b.ins().iadd(text, index);
        self.b.ins().istore8(MemFlags::new(), byte, address, 0);
    }

    fn return_if(&mut self, cond: Value, values: &[Value]) {
        let ret = self.b.create_block();
        let next = self.b.create_block();
        self.b.ins().brif(cond, ret, &[], next, &[]);
        self.b.switch_to_block(ret);
        self.b.ins().return_(values);
        self.b.switch_to_block(next);
    }

    fn when(&mut self, cond: Value, then: impl FnOnce(&mut Self)) {
        let inside = self.b.create_block();
        let done = self.b.create_block();
        self.b.ins().brif(cond, inside, &[], done, &[]);
        self.b.switch_to_block(inside);
        then(self);
        self.b.ins().jump(done, &[]);
        self.b.switch_to_block(done);
    }

    fn while_(&mut self, cond: impl FnOnce(&mut Self) -> Value, body: impl FnOnce(&mut Self)) {
        let header = self.b.create_block();
        let inside = self.b.create_block();
        let done = self.b.create_block();
        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(header);
        let cond = cond(self);
        self.b.ins().brif(cond, inside, &[], done, &[]);
        self.b.switch_to_block(inside);
        body(self);
        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(done);
    }

    /// Run `body` with an index counting from 0 up to `n`.
    fn each(&mut self, n: Value, body: impl FnOnce(&mut Self, Value)) {
        let zero = self.c(0);
        let index = self.var(zero);
        self.while_(
            |f| {
                let i = f.b.use_var(index);
                f.b.ins().icmp(IntCC::SignedLessThan, i, n)
            },
            |f| {
                let i = f.b.use_var(index);
                body(f, i);
                let i = f.b.ins().iadd_imm(i, 1);
                f.b.def_var(index, i);
            },
        );
    }

    /// A pointer to `word` as a heap integer.
    /// A small integer is written to the stack in that same layout, so every operation reads one shape.
    fn view(&mut self, word: Value) -> Value {
        let slot = self.b.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            LIMBS as u32 + 8,
            3,
        ));
        let p = self.b.ins().stack_addr(I64, slot, 0);
        let value = self.b.ins().sshr_imm(word, 1);
        let negative = self.b.ins().icmp_imm(IntCC::SignedLessThan, value, 0);
        let negated = self.b.ins().ineg(value);
        let magnitude = self.b.ins().select(negative, negated, value);
        let high = self.b.ins().ushr_imm(magnitude, 32);
        let sign = self.b.ins().uextend(I64, negative);
        let has_low = self.b.ins().icmp_imm(IntCC::NotEqual, magnitude, 0);
        let has_low = self.b.ins().uextend(I64, has_low);
        let has_high = self.b.ins().icmp_imm(IntCC::NotEqual, high, 0);
        let has_high = self.b.ins().uextend(I64, has_high);
        let len = self.b.ins().iadd(has_low, has_high);
        self.store(sign, p, SIGN);
        self.store(len, p, LEN);
        self.b.ins().istore32(MemFlags::new(), magnitude, p, LIMBS);
        self.b.ins().istore32(MemFlags::new(), high, p, LIMBS + 4);

        let heap = self.b.ins().iadd_imm(word, -1);
        let is_heap = self.b.ins().band_imm(word, 1);
        self.b.ins().select(is_heap, heap, p)
    }
}
