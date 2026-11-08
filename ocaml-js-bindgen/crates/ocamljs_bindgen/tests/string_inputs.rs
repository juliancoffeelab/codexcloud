use ocamljs_bindgen::{abi::Argument, host::Borrowable, numeric, tasks::*, text, *};
#[ocamljs_bindgen]
pub type Text = String;
#[ocamljs_bindgen]
pub type TextSlice = str;
#[ocamljs_bindgen]
pub fn combined(a: Text, b: &TextSlice) -> u32 {
    (a.len() + b.len()) as u32
}
#[ocamljs_bindgen]
pub async fn delayed(value: &TextSlice) -> u32 {
    yield_now().await;
    value.len() as u32
}
unsafe fn prepare(value: &[u8], capacity: usize) -> usize {
    let p = text::__ocamljs_string_alloc(capacity);
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), p as *mut u8, value.len());
    }
    p
}
#[test]
fn owned_alias_reuses_initialized_prefix_and_exact_capacity() {
    for value in ["", "id", "\u{feff}λ🦀\0"] {
        unsafe {
            let before = __ocamljs_live();
            let cap = value.len() + 17;
            let p = prepare(value.as_bytes(), cap);
            let guard = text::Owned::adopt(p, value.len(), cap, (), (), (), (), ());
            let result = text::Owned::lift(guard, p, value.len(), cap, (), (), (), (), ()).unwrap();
            assert_eq!(result, value);
            assert_eq!(result.as_ptr() as usize, p);
            assert_eq!(result.capacity(), cap);
            drop(result);
            assert_eq!(__ocamljs_live(), before);
        }
    }
}
#[test]
fn alias_borrows_and_owned_arguments_transfer_together() {
    unsafe {
        let before = __ocamljs_live();
        let p = prepare(b"abc", 8);
        let q = prepare("λ".as_bytes(), 13);
        assert_eq!(
            __ocamljs_fn_combined(p, 3, 8, (), (), (), (), (), q, 2, 13),
            5
        );
        assert_eq!(__ocamljs_live(), before);
    }
}
#[test]
fn borrowed_alias_survives_suspension_and_cancellation() {
    unsafe {
        for cancel in [false, true] {
            let before = __ocamljs_live();
            let p = prepare("🦀".as_bytes(), 19);
            let task = __ocamljs_fn_delayed(p, 4, 19);
            assert_eq!(poll_task(task).unwrap(), None);
            if cancel {
                assert!(cancel_task(task));
            } else {
                poll_task(task).unwrap().unwrap();
                abi::flat_output(|bytes| {
                    assert_eq!(u32::from_le_bytes(bytes[..4].try_into().unwrap()), 4);
                });
            }
            assert_eq!(__ocamljs_live(), before);
        }
    }
}
#[test]
fn checked_bad_utf8_and_uninitialized_tail_rollback() {
    unsafe {
        let before = __ocamljs_live();
        let p = prepare(&[], 32);
        text::__ocamljs_string_free(p, 32);
        assert_eq!(__ocamljs_live(), before);
        if PARANOID {
            let p = prepare(&[255], 16);
            let guard = <str as Borrowable>::adopt(p, 1, 16);
            assert!(<str as Borrowable>::with_shared(p, 1, 16, guard, |_| ()).is_err());
            assert_eq!(__ocamljs_live(), before);
        }
        assert!(numeric::allocate::<u8>(usize::MAX).is_err());
    }
}
