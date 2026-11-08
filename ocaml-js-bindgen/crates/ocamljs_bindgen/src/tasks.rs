//! Local, bounded task registry; native wakers contain no thread-affine state.
use crate::{
    BoundaryError, BoundaryResult, boundary,
    scoped::{CompletionFuture, LocalFuture, ScopedFuture},
};
use std::{
    any::Any,
    cell::RefCell,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};
const MAX_TASKS: usize = 128;
enum State {
    Empty,
    Live(Task),
    Polling,
}
struct Slot {
    generation: u32,
    state: State,
}
struct Task {
    future: ScopedFuture,
    signal: Arc<Signal>,
}
#[derive(Default)]
struct Registry {
    slots: Vec<Slot>,
    free: Vec<usize>,
    live: usize,
}
thread_local! { static TASKS: RefCell<Registry> = RefCell::new(Registry::default()); }
struct Signal {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    id: u32,
    ready: AtomicBool,
    active: AtomicBool,
}
impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        if self.active.load(Ordering::Acquire) && !self.ready.swap(true, Ordering::AcqRel) {
            // Non-shared wasm32 has one execution thread. This import only queues a
            // later host turn, never calls back into WASM. Native Wake stays purely
            // atomic and can safely be invoked from another thread.
            #[cfg(target_arch = "wasm32")]
            unsafe {
                task_wake(self.id);
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn task_wake(id: u32);
}
impl Registry {
    fn index(&self, id: u32) -> Option<usize> {
        let i = (id & 0xfffff).checked_sub(1)? as usize;
        let s = self.slots.get(i)?;
        (s.generation == id >> 20 && !matches!(s.state, State::Empty)).then_some(i)
    }
    fn retire(&mut self, id: u32) {
        let i = self.index(id).expect("task reservation");
        let s = &mut self.slots[i];
        s.state = State::Empty;
        if s.generation < 0xfff {
            s.generation += 1;
            self.free.push(i);
        }
        self.live -= 1;
    }
}
/// A construction tombstone counts against capacity before user decoding/building.
pub struct TaskReservation {
    id: u32,
    active: bool,
    local: std::marker::PhantomData<std::rc::Rc<()>>,
}
pub fn reserve_task() -> BoundaryResult<TaskReservation> {
    TASKS.with_borrow_mut(|r| {
        if r.live >= MAX_TASKS || (r.free.is_empty() && r.slots.len() >= 0xfffff) {
            return Err(BoundaryError("task limit exceeded"));
        }
        let i = r.free.pop().unwrap_or_else(|| {
            r.slots.push(Slot {
                generation: 1,
                state: State::Empty,
            });
            r.slots.len() - 1
        });
        let id = (r.slots[i].generation << 20) | (i as u32 + 1);
        r.slots[i].state = State::Polling;
        r.live += 1;
        Ok(TaskReservation {
            id,
            active: true,
            local: std::marker::PhantomData,
        })
    })
}
impl TaskReservation {
    pub fn spawn_scoped<A: Any, F>(self, anchors: A, build: F) -> BoundaryResult<u32>
    where
        F: for<'a> FnOnce(&'a mut A) -> LocalFuture<'a>,
    {
        self.spawn_completion(anchors, |anchors| {
            let future = build(anchors);
            Box::pin(async move { future.await.map(crate::abi::Completion::Direct) })
        })
    }
    pub fn spawn_completion<A: Any, F>(mut self, anchors: A, build: F) -> BoundaryResult<u32>
    where
        F: for<'a> FnOnce(&'a mut A) -> CompletionFuture<'a>,
    {
        let future = ScopedFuture::new(anchors, build);
        let signal = Arc::new(Signal {
            id: self.id,
            ready: AtomicBool::new(true),
            active: AtomicBool::new(true),
        });
        TASKS.with_borrow_mut(|r| {
            let i = r.index(self.id).expect("construction reservation");
            r.slots[i].state = State::Live(Task { future, signal });
        });
        self.active = false;
        Ok(self.id)
    }
}
impl Drop for TaskReservation {
    fn drop(&mut self) {
        if self.active {
            TASKS.with_borrow_mut(|r| r.retire(self.id));
        }
    }
}
pub fn spawn_scoped<A: Any, F>(anchors: A, build: F) -> BoundaryResult<u32>
where
    F: for<'a> FnOnce(&'a mut A) -> LocalFuture<'a>,
{
    reserve_task()?.spawn_scoped(anchors, build)
}
pub fn poll_task(id: u32) -> BoundaryResult<Option<u32>> {
    let mut task = TASKS.with_borrow_mut(|r| {
        let i = r.index(id).ok_or(BoundaryError("invalid task"))?;
        match std::mem::replace(&mut r.slots[i].state, State::Polling) {
            State::Live(t) => Ok(t),
            state => {
                r.slots[i].state = state;
                Err(BoundaryError("task is already polling"))
            }
        }
    })?;
    task.signal.ready.store(false, Ordering::Release);
    let waker = Waker::from(task.signal.clone());
    match task.future.poll(&mut Context::from_waker(&waker)) {
        Poll::Pending => {
            TASKS.with_borrow_mut(|r| {
                let i = r.index(id).unwrap();
                r.slots[i].state = State::Live(task);
            });
            Ok(None)
        }
        Poll::Ready(value) => {
            task.signal.active.store(false, Ordering::Release);
            // Polling tombstone prevents identity reuse during arbitrary user Drop.
            drop(task);
            TASKS.with_borrow_mut(|r| r.retire(id));
            value.and_then(|value| value.publish().map(Some))
        }
    }
}
pub fn cancel_task(id: u32) -> bool {
    let task = TASKS.with_borrow_mut(|r| {
        let i = r.index(id)?;
        match std::mem::replace(&mut r.slots[i].state, State::Polling) {
            State::Live(t) => Some(t),
            state => {
                r.slots[i].state = state;
                None
            }
        }
    });
    if let Some(task) = task {
        task.signal.active.store(false, Ordering::Release);
        drop(task);
        TASKS.with_borrow_mut(|r| r.retire(id));
        true
    } else {
        false
    }
}
pub fn task_count() -> usize {
    TASKS.with_borrow(|r| r.live)
}
pub extern "C" fn __ocamljs_task_poll(id: u32) -> u32 {
    boundary(poll_task(id)).unwrap_or(0)
}
pub extern "C" fn __ocamljs_task_cancel(id: u32) -> u32 {
    cancel_task(id) as u32
}
#[cfg_attr(
    any(
        not(target_arch = "wasm32"),
        feature = "resource-diagnostics",
        ocamljs_resource_diagnostics
    ),
    unsafe(no_mangle)
)]
pub extern "C" fn __ocamljs_tasks() -> usize {
    task_count()
}
#[cfg_attr(
    any(
        not(target_arch = "wasm32"),
        feature = "resource-diagnostics",
        ocamljs_resource_diagnostics
    ),
    unsafe(no_mangle)
)]
pub extern "C" fn __ocamljs_task_ready(id: u32) -> u32 {
    TASKS.with_borrow(|r| {
        r.index(id)
            .and_then(|i| match &r.slots[i].state {
                State::Live(t) => Some(t.signal.ready.load(Ordering::Acquire) as u32),
                _ => None,
            })
            .unwrap_or(0)
    })
}
/// A real Pending/wake/Ready future. The host schedules its wake on a later task.
#[derive(Default)]
pub struct YieldNow(bool);
impl Future for YieldNow {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
pub fn yield_now() -> YieldNow {
    YieldNow::default()
}
