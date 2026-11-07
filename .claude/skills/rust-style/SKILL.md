---
name: rust-style
description: House style for writing and editing Rust in this repo. Load before writing or changing any `.rs` file, including tests. Covers chunking with blank lines, import grouping and full qualification, avoiding macros, replacing comments with small named functions, and flattening nested code.
---

# Rust house style

Apply every rule below to new code and to code you touch.
When editing, bring the lines you change in line with these rules, but do not restyle unrelated code in the same change.

## 1. Chunk code by concept

Lines that belong together sit next to each other.
A new concept starts after one empty line.
Never put two empty lines in a row, and never leave a lone line between two blank lines unless it is a concept of its own.

```rust
pub fn gen_class(ast: &AST, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained {
    let Node::Class { body: Some(body), ty, args, .. } = &ast.node else {
        return Ok(env.clone());
    };

    let name = Name::from(&StringName::try_from(ty)?);
    let env = bind_self(env, ty, &name);

    let env = gen_vec(args, &env, ctx, constr)?;
    gen_body(body, &env, ctx, constr)
}
```

Destructuring is one chunk, binding `self` is another, generating children is a third.

## 2. Import order and grouping

Imports come in this order, with one empty line between each group:

1. `std`, `core` and `alloc`.
2. External crates, such as `log`, `itertools` or `ruff_python_parser`.
3. Our own crates, starting with `crate::`, then workspace crates such as `tests_util`.
4. Module declarations: private `mod` first, then `pub mod`.

```rust
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use log::{info, trace};

use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

mod constrain;
mod ident;

pub mod ast;
pub mod context;
```

Keep each group sorted alphabetically, as `cargo fmt` does.

## 3. Fully qualify imports

Import with the full path from `crate::`, never relative to the current module.

```rust
// Wrong
use super::env::Environment;
use self::builder::ConstrBuilder;

// Right
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::constraint::builder::ConstrBuilder;
```

The one exception is a utility scope, such as a `#[cfg(test)] mod test`, which may `use super::Item` to reach the item under test.
Import the item by name rather than `use super::*`.

Prefer importing the type or function you use over importing its parent module and writing `module::item` at each call site.

## 4. Do not write macros

Do not declare `macro_rules!` or procedural macros.
A function, a generic, a trait or a plain `match` almost always does the job and stays readable and debuggable.
Only reach for a macro when it removes a large, real, repeated pattern that no function can express, and say why in its rustdoc.
Saving a handful of lines is never reason enough.

Standard macros such as `format!`, `vec!`, `matches!`, `assert_eq!` and `#[test_case(...)]` are fine.

## 5. Name things instead of commenting them

Avoid code comments.
If a block needs a comment to explain what it does, extract it into a short function whose name says it, and put the explanation in that function's rustdoc.

```rust
// Wrong
// Bind self so field initializers can reach class arguments via `self.<name>`.
let var = AST::new(ty.pos, Id { lit: String::from(SELF) });
let env = env.insert_var(false, &id_from_var(&var)?, &name);

// Right
let env = bind_self(env, ty, &name)?;

/// Binds `self` to the class type, so field initializers can reach class arguments as `self.<name>`.
fn bind_self(env: &Environment, ty: &AST, name: &Name) -> Result<Environment, Vec<TypeErr>> {
    let var = AST::new(ty.pos, Id { lit: String::from(SELF) });
    Ok(env.insert_var(false, &id_from_var(&var)?, name))
}
```

Rustdoc follows the repo's markdown rules:

- One sentence per line, never wrapped.
- Short sentences, usually one to three.
- Say what the item is for and why, not how it was arrived at.

A function name earns its place when it names a concept or removes nesting.
Do not wrap a single trivial expression just to give it a name.
A remaining comment is acceptable only when it records something the code cannot, such as a link to an upstream bug.

## 6. Keep code flat

Avoid nesting.
Aim for at most two levels of indentation inside a function body.
Reach for these, in roughly this order:

- `let ... else` to destructure or bail out early, instead of wrapping the rest of the function in a `match` or `if let`.
- `?` to propagate errors, instead of matching on `Result` or `Option`.
- Guard clauses with early `return` or `continue`, instead of `if` around the happy path.
- Matching on a tuple, `match (a, b)`, instead of a `match` inside a `match` arm.
- Iterator combinators such as `map`, `filter_map`, `flat_map` and `collect::<Result<_, _>>()`, instead of loops with nested conditionals.
- Extracting the inner block into a named function, per section 5.

```rust
// Wrong
match &ast.node {
    Node::Class { body: Some(body), .. } => match &body.node {
        Node::Block { statements } => {
            for stmt in statements {
                if is_field(stmt) {
                    // ...
                }
            }
        }
        _ => Ok(env.clone()),
    },
    _ => Ok(env.clone()),
}

// Right
let Node::Class { body: Some(body), .. } = &ast.node else {
    return Ok(env.clone());
};
let Node::Block { statements } = &body.node else {
    return Ok(env.clone());
};

statements.iter().filter(|stmt| is_field(stmt)).try_fold(env.clone(), |env, field| gen_field(field, &env, ctx, constr))
```

## Before finishing

Re-read every function you wrote or changed and check:

- Are concepts separated by single blank lines, and related lines kept together?
- Are imports grouped std, external, crate, then `mod` declarations, and fully qualified?
- Is there any `super::` outside a test or utility module?
- Did you add a macro?
- Did you add a comment that could be a function name and rustdoc instead?
- Is anything nested more than two levels deep that a `let ... else`, `?` or extracted function would flatten?
