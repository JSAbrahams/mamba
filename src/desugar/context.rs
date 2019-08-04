use crate::core::construct::Core;

pub struct State {
    pub tup:         usize,
    pub expect_expr: bool,
    pub interface:   bool
}

impl State {
    pub fn new() -> State { State { tup: 1, expect_expr: false, interface: false } }
}

pub struct Imports {
    pub imports: Vec<Core>
}

impl Imports {
    pub fn new() -> Imports { Imports { imports: vec![] } }

    pub fn add_import(&mut self, import: &str) {
        self.imports.push(Core::Import { imports: vec![Core::Id { lit: String::from(import) }] });
    }

    pub fn add_from_import(&mut self, from: &str, import: &str) {
        self.imports.push(Core::FromImport {
            from:   Box::from(Core::Id { lit: String::from(from) }),
            import: Box::from(Core::Import {
                imports: vec![Core::Id { lit: String::from(import) }]
            })
        });
    }
}
