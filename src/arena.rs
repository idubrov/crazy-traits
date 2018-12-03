trait Object {
    fn type_name(&self) -> &str;
    fn as_string(&self) -> &String;
}

struct Wrapper {
    value: *const String,
    type_name: String,
}

impl Object for Wrapper {
    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn as_string(&self) -> &String {
        unsafe { &*self.value }
    }
}

struct TypeSystem {
    wrappers: typed_arena::Arena<Wrapper>,
}

impl TypeSystem {
    pub fn new() -> Self {
        Self {
            wrappers: typed_arena::Arena::new(),
        }
    }

    pub fn annotate<'a: 'b, 'b>(&'a self, input: &'b String, type_name: &str) -> &'b dyn Object {
        self.wrappers.alloc(Wrapper {
            value: input,
            type_name: type_name.into(),
        })
    }
}

#[test]
fn test() {
    let ts = TypeSystem::new();

    let input: String = "hello".into();
    let annotated1 = ts.annotate(&input, "Widget");
    let annotated2 = ts.annotate(&input, "Gadget");

    assert_eq!("Widget", annotated1.type_name());
    assert_eq!("Gadget", annotated2.type_name());

    let unwrapped1 = annotated1.as_string();
    let unwrapped2 = annotated2.as_string();

    assert_eq!(unwrapped1 as *const String, &input as *const String);
    assert_eq!(unwrapped2 as *const String, &input as *const String);
}
