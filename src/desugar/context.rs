pub struct State {
    pub in_tup: bool
}

impl State {
    pub fn new() -> State { State { in_tup: false } }
}

pub struct Context {
    fields:    Vec<String>,
    functions: Vec<String>,
    classes:   Vec<String>
}

impl Context {
    pub fn new() -> Context {
        Context { fields: Vec::new(), functions: Vec::new(), classes: Vec::new() }
    }

    pub fn contains_field(self, field: String) -> bool { self.fields.contains(&field) }

    pub fn contains_function(self, function: String) -> bool { self.functions.contains(&function) }

    pub fn contains_class(self, class: String) -> bool { self.classes.contains(&class) }
}
