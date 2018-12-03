trait Object {
    fn type_name(&self) -> &str;
    fn as_string(&self) -> &String;
}

struct Wrapper<'a> {
    value: &'a String,
    type_name: String,
}

impl<'a> Object for Wrapper<'a> {
    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn as_string(&self) -> &String {
        self.value
    }
}

fn annotate<'a>(input: &'a String, type_name: &str) -> &'a dyn Object {
    let b = Box::new(Wrapper {
        value: input,
        type_name: type_name.into(),
    });
    Box::leak(b)
}

#[test]
fn test() {
    let input: String = "hello".into();
    let annotated1 = annotate(&input, "Widget");
    let annotated2 = annotate(&input, "Gadget");

    assert_eq!("Widget", annotated1.type_name());
    assert_eq!("Gadget", annotated2.type_name());

    let unwrapped1 = annotated1.as_string();
    let unwrapped2 = annotated2.as_string();

    assert_eq!(unwrapped1 as *const String, &input as *const String);
    assert_eq!(unwrapped2 as *const String, &input as *const String);
}
