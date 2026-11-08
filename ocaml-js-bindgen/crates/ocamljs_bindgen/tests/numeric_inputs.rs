use ocamljs_bindgen::{numeric, tasks::*, *};
use std::{mem::align_of, ptr};
unsafe fn prepare<T: numeric::Element>(values: &[T]) -> usize {
    let pointer = numeric::allocate::<T>(values.len()).unwrap();
    assert_eq!(pointer % align_of::<T>(), 0);
    unsafe { ptr::copy_nonoverlapping(values.as_ptr(), pointer as *mut T, values.len()) };
    pointer
}
#[ocamljs_bindgen]
pub fn numeric_pair(left: Vec<f64>, right: &[f64], bias: f64) -> f64 {
    left.iter().chain(right).sum::<f64>() + bias
}
type FloatSlice = [f64];
#[ocamljs_bindgen]
pub async fn delayed_numeric(values: &FloatSlice) -> f64 {
    yield_now().await;
    values.iter().sum()
}
#[test]
fn direct_vector_lift_preserves_allocation_and_float_bits() {
    unsafe {
        let values = [-0.0, f64::from_bits(0x7ff8000000000123), f64::INFINITY];
        let pointer = prepare(&values);
        let guard = <abi::Vector<f64> as abi::Argument<Vec<f64>>>::adopt(
            pointer,
            3,
            (),
            (),
            (),
            (),
            (),
            (),
        );
        let value = <abi::Vector<f64> as abi::Argument<Vec<f64>>>::lift(
            guard,
            pointer,
            3,
            (),
            (),
            (),
            (),
            (),
            (),
        )
        .unwrap();
        assert_eq!(value.as_ptr() as usize, pointer);
        assert_eq!(
            value.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            values.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
    }
}
#[test]
fn mixed_owned_borrowed_and_scalar_arguments_transfer_together() {
    unsafe {
        let before = __ocamljs_live();
        let a = prepare(&[1.0, 2.0]);
        let b = prepare(&[3.0, 4.0]);
        assert_eq!(__ocamljs_live(), before + 2);
        assert_eq!(
            __ocamljs_fn_numeric_pair(a, 2, (), (), (), (), (), (), b, 2, (), 0.5),
            10.5
        );
        assert_eq!(__ocamljs_live(), before);
    }
}
#[test]
fn aliased_slice_storage_survives_suspension_and_cancellation() {
    unsafe {
        for cancel in [false, true] {
            let before = __ocamljs_live();
            let pointer = prepare(&[17.0, 4.0]);
            let task = __ocamljs_fn_delayed_numeric(pointer, 2, ());
            assert_eq!(poll_task(task).unwrap(), None);
            if cancel {
                assert!(cancel_task(task));
            } else {
                poll_task(task).unwrap().unwrap();
                // Scalar completion uses the shared return slot, not an arena ID.
                abi::flat_output(|bytes| {
                    assert_eq!(f64::from_le_bytes(bytes[..8].try_into().unwrap()), 21.0);
                });
            }
            assert_eq!(__ocamljs_live(), before);
        }
    }
}
#[test]
fn empty_and_overflowing_allocations_obey_layout_contract() {
    unsafe {
        let before = __ocamljs_live();
        let pointer = prepare::<f64>(&[]);
        assert_ne!(pointer, 0);
        let value = numeric::take_vec::<f64>(pointer, 0);
        assert!(value.is_empty());
        drop(value);
        assert!(numeric::allocate::<f64>(usize::MAX).is_err());
        assert_eq!(__ocamljs_live(), before);
    }
}
