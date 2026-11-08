//! Owned host registrations. Host exceptions return status, never unwind Rust.
use crate::{__ocamljs_output_free, BoundaryError, BoundaryResult, Encode, publish};
use std::marker::PhantomData;
pub struct Callback<T> {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    reference: crate::host::Ref,
    marker: PhantomData<(fn(T), std::rc::Rc<()>)>,
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn callback_call(id: u32, output: u32) -> u32;
}
impl<T> Callback<T> {
    pub fn acquire(id: u32) -> BoundaryResult<Self> {
        Ok(Self {
            reference: unsafe { crate::host::Ref::from_borrowed(id) },
            marker: PhantomData,
        })
    }
}
impl<T: Encode> Callback<T> {
    pub fn emit(&self, event: T) -> BoundaryResult<()> {
        let output = publish(event)?;
        #[cfg(target_arch = "wasm32")]
        let status = unsafe { callback_call(self.reference.id(), output) };
        #[cfg(not(target_arch = "wasm32"))]
        let status = 1;
        __ocamljs_output_free(output);
        if status == 0 {
            Ok(())
        } else {
            Err(BoundaryError("callback failed or removed"))
        }
    }
}
