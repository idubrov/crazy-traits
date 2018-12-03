use dynasmrt::DynasmApi;
use std::cell::RefCell;
use std::collections::HashMap;

/// API interface of the object, we want our data (`String`s) to comply to it!
trait Object {
    fn type_name(&self) -> &str;
    fn as_string(&self) -> &String;
}

/// Trait object runtime representation
#[repr(C)]
struct TraitObject {
    pub data: *const (),
    pub vtable: *const (),
}

/// Every virtual table starts with these entries. We use arbitrary empty trait ([`Whatever`])
/// to capture these entries.
#[repr(C)]
#[derive(Clone, Copy)]
struct VirtualTableHeader {
    destructor_fn: fn(*mut ()),
    size: usize,
    align: usize,
}

/// Virtual table representation for the Object trait
#[repr(C)]
struct ObjectVirtualTable {
    header: VirtualTableHeader,
    type_name_fn: fn(*const String) -> *const str,
    as_string_fn: fn(*const String) -> *const String,
}

/// Trait used to capture destructor, align and size of our object
trait Whatever {}

impl<T> Whatever for T {}

/// Keeps our generated vtable
struct TypeInfo {
    vtable: ObjectVirtualTable,
    #[allow(unused)]
    buffer: dynasmrt::ExecutableBuffer,
}

#[derive(Default)]
struct TypeSystem {
    infos: RefCell<HashMap<String, TypeInfo>>,
}

impl TypeSystem {
    pub fn annotate<'a: 'b, 'b>(&'a self, input: &'b String, type_name: &str) -> &'b dyn Object {
        let type_name = type_name.to_string();
        let type_name_ptr = type_name.as_str().as_ptr();
        let type_name_len = type_name.as_str().len();
        let mut infos = self.infos.borrow_mut();
        let imp = infos.entry(type_name).or_insert_with(|| unsafe {
            let mut ops = dynasmrt::x64::Assembler::new().unwrap();
            let type_name_offset = ops.offset();
            dynasm!(ops
                ; mov rax, QWORD type_name_ptr as i64
                ; mov rdx, QWORD type_name_len as i64
                ; ret
            );

            let as_string_offset = ops.offset();
            dynasm!(ops
                ; mov rax, rdi
                ; ret
            );
            let buffer = ops.finalize().unwrap();

            let whatever = input as &dyn Whatever;
            let whatever_obj = std::mem::transmute::<&dyn Whatever, TraitObject>(whatever);
            let whatever_vtable_header = whatever_obj.vtable as *const VirtualTableHeader;
            let vtable = ObjectVirtualTable {
                header: *whatever_vtable_header,
                type_name_fn: std::mem::transmute(buffer.ptr(type_name_offset)),
                as_string_fn: std::mem::transmute(buffer.ptr(as_string_offset)),
            };

            TypeInfo { vtable, buffer }
        });

        assert_eq!(imp.vtable.header.size, std::mem::size_of::<String>());
        assert_eq!(imp.vtable.header.align, std::mem::align_of::<String>());

        let object_obj = TraitObject {
            data: input as *const String as *const (),
            vtable: &imp.vtable as *const ObjectVirtualTable as *const (),
        };
        unsafe { std::mem::transmute::<TraitObject, &dyn Object>(object_obj) }
    }
}

#[test]
fn test() {
    let ts = TypeSystem::default();

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
