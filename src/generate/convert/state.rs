use crate::generate::ast::node::Core;
use crate::generate::GenArguments;

#[derive(Clone, Debug)]
pub struct State {
    pub tup: usize,
    pub interface: bool,
    pub expand_ty: bool,
    pub def_as_fun_arg: bool,
    pub tup_lit: bool,
    pub assign_to: Option<Core>,

    pub annotate: bool,
}

impl From<&GenArguments> for State {
    fn from(gen_arguments: &GenArguments) -> Self {
        State { annotate: gen_arguments.annotate, ..State::new() }
    }
}

impl State {
    pub fn new() -> State {
        State {
            tup: 1,
            interface: false,
            expand_ty: true,
            def_as_fun_arg: false,
            tup_lit: false,
            assign_to: None,
            annotate: false,
        }
    }

    pub fn in_tup(&self, tup: usize) -> State {
        State { tup, ..self.clone() }
    }

    pub fn tuple_literal(&self) -> State {
        State { tup_lit: true, ..self.clone() }
    }

    pub fn in_interface(&self, interface: bool) -> State {
        State { interface, ..self.clone() }
    }

    pub fn expand_ty(&self, expand_ty: bool) -> State {
        State { expand_ty, ..self.clone() }
    }

    pub fn def_as_fun_arg(&self, def_as_fun_arg: bool) -> State {
        State { def_as_fun_arg, ..self.clone() }
    }

    pub fn assign_to(&self, assign_to: Option<&Core>) -> State {
        State { assign_to: assign_to.cloned(), ..self.clone() }
    }
}

pub struct Imports {
    pub imports: Vec<Core>,
}

impl Default for Imports {
    fn default() -> Self {
        Self::new()
    }
}

impl Imports {
    pub fn new() -> Imports {
        Imports { imports: vec![] }
    }

    pub fn add_import(&mut self, import: &str) {
        let import = Core::Import { from: None, import: vec![Core::Id { lit: String::from(import) }], alias: vec![] };
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }

    pub fn add_from_import(&mut self, from: &str, import: &str) {
        let import = Core::Import {
            from: Some(Box::from(Core::Id { lit: String::from(from) })),
            import: vec![Core::Id { lit: String::from(import) }],
            alias: vec![],
        };

        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }
}
