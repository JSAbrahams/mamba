use crate::core::construct::Core;

// TODO remove expect_expr once type checker augments AST
pub struct State {
    pub tup:         usize,
    pub interface:   bool,
    pub expand_ty:   bool,
    pub expect_expr: bool,
    pub expect_ret:  bool
}

impl State {
    pub fn new() -> State {
        State {
            tup:         1,
            interface:   false,
            expand_ty:   true,
            expect_expr: false,
            expect_ret:  false
        }
    }

    pub fn expect_return(&self, expect_ret: bool) -> State { State { expect_ret, ..*self.clone() } }

    pub fn expect_expr(&self, expect_expr: bool) -> State { State { expect_expr, ..*self.clone() } }

    pub fn in_tup(&self, tup: usize) -> State { State { tup, ..*self.clone() } }

    pub fn in_interface(&self, interface: bool) -> State { State { interface, ..*self.clone() } }

    pub fn expand_ty(&self, expand_ty: bool) -> State { State { expand_ty, ..*self.clone() } }
}

pub struct Imports {
    pub imports: Vec<Core>
}

impl Imports {
    pub fn new() -> Imports { Imports { imports: vec![] } }

    pub fn add_import(&mut self, import: &str) {
        let import = Core::Import { imports: vec![Core::Id { lit: String::from(import) }] };
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }

    pub fn add_from_import(&mut self, from: &str, import: &str) {
        let import = Core::FromImport {
            from:   Box::from(Core::Id { lit: String::from(from) }),
            import: Box::from(Core::Import {
                imports: vec![Core::Id { lit: String::from(import) }]
            })
        };

        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }
}
