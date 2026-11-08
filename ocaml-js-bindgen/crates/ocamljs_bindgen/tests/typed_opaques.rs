use ocamljs_bindgen::{scoped::SharedAnchor, *};
use std::{cell::Cell, rc::Rc};

#[ocamljs_bindgen(opaque)]
struct Value(u32);

struct Large([u8; 65536]);
opaque_storage!(Large);
#[test]
fn large_inline_objects_do_not_construct_a_whole_page_on_the_stack() {
    unsafe {
        let id = insert(Large([17; 65536]));
        with_ref::<Large, _>(id, |value| assert!(value.0.iter().all(|byte| *byte == 17))).unwrap();
        dispose(id).unwrap();
    }
}

#[test]
fn root_opaque_is_a_direct_handle_but_nested_owners_remain_transactional() {
    unsafe {
        let baseline = __ocamljs_live();
        let id = abi::lower_output(Value(7)).unwrap();
        assert_eq!(__ocamljs_live(), baseline + 1);
        assert_eq!(__ocamljs_contains(id), 1);
        assert_eq!(with_ref::<Value, _>(id, |v| v.0).unwrap(), 7);
        dispose(id).unwrap();
        let packet = publish(vec![Value(8), Value(9)]).unwrap();
        assert_eq!(__ocamljs_live(), baseline + 3);
        __ocamljs_output_free(packet);
        assert_eq!(__ocamljs_live(), baseline);
    }
}

#[test]
fn synchronous_and_suspended_borrows_stay_stable_across_reentrant_page_growth() {
    unsafe {
        let id = insert(Value(7));
        let shared = SharedAnchor::<Value>::new(id).unwrap();
        let pointer = shared.get() as *const Value;
        with_ref::<Value, _>(id, |value| {
            assert_eq!(value as *const Value, pointer);
            let others = (0..1024).map(|n| insert(Value(n))).collect::<Vec<_>>();
            assert_eq!(value.0, 7);
            assert!(dispose(id).is_err());
            assert!(with_mut::<Value, _>(id, |_| ()).is_err());
            for other in others {
                dispose(other).unwrap();
            }
        })
        .unwrap();
        drop(shared);
        with_mut::<Value, _>(id, |value| {
            let others = (0..1024).map(|n| insert(Value(n))).collect::<Vec<_>>();
            value.0 += 1;
            assert_eq!(value as *const Value, pointer);
            for other in others {
                dispose(other).unwrap();
            }
        })
        .unwrap();
        dispose(id).unwrap();
    }
}

struct Reentrant {
    drops: Rc<Cell<u32>>,
    spawn: bool,
    sentinel: u32,
}
opaque_storage!(Reentrant);
impl Drop for Reentrant {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        if self.spawn {
            let id = insert(Reentrant {
                drops: self.drops.clone(),
                spawn: false,
                sentinel: 99,
            });
            unsafe {
                dispose(id).unwrap();
            }
            assert_eq!(
                self.sentinel, 17,
                "reused cell must not overwrite the destructor's value"
            );
        }
    }
}
#[test]
fn same_type_destructor_reentry_uses_independent_moved_out_storage() {
    unsafe {
        let drops = Rc::new(Cell::new(0));
        let id = insert(Reentrant {
            drops: drops.clone(),
            spawn: true,
            sentinel: 17,
        });
        dispose(id).unwrap();
        assert_eq!(drops.get(), 2);
        assert_eq!(__ocamljs_contains(id), 0);
    }
}

struct Retainer {
    child: SharedAnchor<Value>,
    drops: Rc<Cell<u32>>,
}
opaque_storage!(Retainer);
impl Drop for Retainer {
    fn drop(&mut self) {
        assert_eq!(self.child.0, 11);
        self.drops.set(self.drops.get() + 1);
    }
}
#[test]
fn independent_allocations_do_not_introduce_page_retention_cycles() {
    unsafe {
        let drops = Rc::new(Cell::new(0));
        let child = insert(Value(11));
        let parent = insert(Retainer {
            child: SharedAnchor::new(child).unwrap(),
            drops: drops.clone(),
        });
        assert!(dispose(child).is_err());

        dispose(parent).unwrap();
        dispose(child).unwrap();

        assert_eq!(drops.get(), 1);
    }
}
