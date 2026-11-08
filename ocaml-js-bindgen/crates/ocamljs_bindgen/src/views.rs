//! Owned view tokens retain storage without exporting Rust references.
use crate::{BoundaryError, BoundaryResult};
use std::{cell::RefCell, rc::Rc};
pub struct ViewOwner<T>(Rc<RefCell<T>>);
pub struct SharedView<T>(Rc<RefCell<T>>);
impl<T> ViewOwner<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
    pub fn view(&self) -> SharedView<T> {
        SharedView(self.0.clone())
    }
    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.0.borrow())
    }
    pub fn mutate<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> BoundaryResult<R> {
        if Rc::strong_count(&self.0) != 1 {
            return Err(BoundaryError("live views prevent mutation"));
        }
        Ok(f(&mut self.0.borrow_mut()))
    }
}
impl<T> SharedView<T> {
    pub fn child(&self) -> Self {
        Self(self.0.clone())
    }
    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.0.borrow())
    }
}
