//! Reusable owned vectors for shared cache entries. A slot becomes writable
//! again only after the cache and every reader have released their handles.
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub(super) struct SharedBuffers<T> {
    slots: Vec<Rc<Vec<T>>>,
    cursor: usize,
    elements: usize,
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub(super) struct SharedBuffers<T>(std::marker::PhantomData<T>);
#[cfg(target_arch = "wasm32")]
pub(super) type Buffers<T> = Rc<std::cell::RefCell<SharedBuffers<T>>>;
#[cfg(not(target_arch = "wasm32"))]
pub(super) type Buffers<T> = SharedBuffers<T>;
impl<T> SharedBuffers<T> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_buffers: usize, _elements: usize) -> Self {
        Self(std::marker::PhantomData)
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn borrow_mut(&self) -> &Self {
        self
    }
    #[cfg(target_arch = "wasm32")]
    pub fn new(buffers: usize, elements: usize) -> Self {
        Self {
            slots: (0..buffers)
                .map(|_| Rc::new(Vec::with_capacity(elements)))
                .collect(),
            cursor: 0,
            elements,
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn fill(&self, write: impl FnOnce(&mut Vec<T>)) -> Rc<Vec<T>> {
        let mut buffer = Vec::new();
        write(&mut buffer);
        Rc::new(buffer)
    }
    #[cfg(target_arch = "wasm32")]
    pub fn fill(&mut self, write: impl FnOnce(&mut Vec<T>)) -> Rc<Vec<T>> {
        let count = self.slots.len();
        let available = (0..count)
            .map(|offset| (self.cursor + offset) % count)
            .find(|&index| {
                Rc::strong_count(&self.slots[index]) == 1
                    && self.slots[index].capacity() >= self.elements
            });
        let index = available.unwrap_or_else(|| {
            self.slots.push(Rc::new(Vec::with_capacity(self.elements)));
            count
        });
        self.cursor = (index + 1) % self.slots.len();
        let buffer = Rc::get_mut(&mut self.slots[index]).expect("unleased buffer");
        buffer.clear();
        write(buffer);
        self.slots[index].clone()
    }
}

// Geometry caches retain live fields; unused backing storage is shared across
// layouts rather than reserving a complete spare inventory for every map.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static POOLS: std::cell::RefCell<rustc_hash::FxHashMap<std::any::TypeId,Box<dyn std::any::Any>>> = Default::default();
}
impl<T: 'static> SharedBuffers<T> {
    pub fn shared(_buffers: usize, _elements: usize) -> Buffers<T> {
        #[cfg(target_arch = "wasm32")]
        {
            POOLS.with_borrow_mut(|pools| {
                pools
                    .entry(std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new(Rc::new(std::cell::RefCell::new(Self::new(0, 0)))))
                    .downcast_ref::<Rc<std::cell::RefCell<Self>>>()
                    .expect("shared buffer type")
                    .clone()
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (_buffers, _elements);
            Self::new(0, 0)
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn prepare(&mut self, buffers: usize, elements: usize) {
        self.elements = self.elements.max(elements);
        let mut free = 0;
        for slot in &mut self.slots {
            if let Some(values) = Rc::get_mut(slot) {
                values.clear();
                values.reserve_exact(self.elements);
                free += 1;
            }
        }
        self.slots.reserve(buffers.saturating_sub(free));
        while free < buffers {
            self.slots.push(Rc::new(Vec::with_capacity(self.elements)));
            free += 1;
        }
    }
}
