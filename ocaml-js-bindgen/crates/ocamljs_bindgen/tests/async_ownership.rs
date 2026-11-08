use ocamljs_bindgen::{
    scoped::{BytesAnchor, MoveAnchor, MutAnchor, SharedAnchor},
    tasks::*,
    *,
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
#[derive(Default)]
struct Value {
    number: u32,
}
ocamljs_bindgen::opaque_storage!(Value);
struct Borrowing<'a> {
    value: &'a mut Value,
    seen: Rc<RefCell<Vec<&'static str>>>,
    stage: u8,
}
impl Future for Borrowing<'_> {
    type Output = BoundaryResult<u32>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.value.number += 1;
        if self.stage == 0 {
            self.stage = 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            self.seen.borrow_mut().push("ready");
            Poll::Ready(publish(self.value.number))
        }
    }
}
impl Drop for Borrowing<'_> {
    fn drop(&mut self) {
        self.value.number += 10;
        self.seen.borrow_mut().push("future dropped");
    }
}
#[test]
fn mut_borrow_survives_pending_and_future_drop_precedes_anchor_release() {
    unsafe {
        for finish in [false, true] {
            let id = insert(Value::default());
            let seen = Rc::new(RefCell::new(vec![]));
            let log = seen.clone();
            let task = spawn_scoped(MutAnchor::<Value>::new(id).unwrap(), move |a| {
                Box::pin(Borrowing {
                    value: a.get_mut(),
                    seen: log,
                    stage: 0,
                })
            })
            .unwrap();
            assert!(with_ref::<Value, _>(id, |_| ()).is_err());
            assert!(dispose(id).is_err());
            assert_eq!(poll_task(task).unwrap(), None);
            assert!(with_mut::<Value, _>(id, |_| ()).is_err());
            if finish {
                let output = poll_task(task).unwrap().unwrap();
                assert_eq!(input_from_output::<u32>(output), 2);
                __ocamljs_output_free(output);
            } else {
                assert!(cancel_task(task));
            }
            assert_eq!(
                with_ref::<Value, _>(id, |v| v.number).unwrap(),
                if finish { 12 } else { 11 }
            );
            assert_eq!(seen.borrow().last(), Some(&"future dropped"));
            assert!(!cancel_task(task));
            dispose(id).unwrap();
        }
    }
}
fn input_from_output<T: Decode>(id: u32) -> T {
    let ptr = __ocamljs_output_ptr(id) as *const u8;
    let len = __ocamljs_output_len(id);
    unsafe {
        decode_segments(
            std::slice::from_raw_parts(ptr, len),
            std::slice::from_raw_parts(
                __ocamljs_output_pool_ptr(id) as *const u8,
                __ocamljs_output_pool_len(id),
            ),
        )
        .unwrap()
    }
}
#[test]
fn shared_borrows_and_input_storage_survive_suspension_without_extra_rust_copy() {
    unsafe {
        let id = insert(Value { number: 42 });
        let input = __ocamljs_alloc(3);
        let ptr = __ocamljs_input_ptr(input) as *mut u8;

        std::ptr::copy_nonoverlapping(b"abc".as_ptr(), ptr, 3);

        let anchors = (
            SharedAnchor::<Value>::new(id).unwrap(),
            SharedAnchor::<Value>::new(id).unwrap(),
            BytesAnchor::new(input).unwrap(),
        );
        let task = spawn_scoped(anchors, |(a, b, bytes)| {
            Box::pin(async move {
                yield_now().await;
                assert_eq!(bytes.text()?, "abc");
                publish(a.get().number + b.get().number)
            })
        })
        .unwrap();
        __ocamljs_input_free(input);
        assert_eq!(poll_task(task).unwrap(), None);
        let output = poll_task(task).unwrap().unwrap();
        assert_eq!(input_from_output::<u32>(output), 84);
        __ocamljs_output_free(output);
        dispose(id).unwrap();
    }
}
#[test]
fn construction_reservation_counts_before_reentrant_build_and_rolls_back() {
    let outer = spawn_scoped((), |_| {
        let mut tasks = vec![];
        for _ in 0..127 {
            tasks.push(spawn_scoped((), |_| Box::pin(std::future::pending())).unwrap());
        }
        assert!(spawn_scoped((), |_| Box::pin(std::future::pending())).is_err());
        assert_eq!(task_count(), 128);
        for id in tasks {
            assert!(cancel_task(id));
        }
        Box::pin(std::future::pending())
    })
    .unwrap();
    assert_eq!(task_count(), 1);
    assert!(cancel_task(outer));
    assert_eq!(task_count(), 0);
    let before = task_count();
    let reservation = reserve_task().unwrap();
    drop(reservation);
    assert_eq!(task_count(), before);
}
struct SaveWake(Arc<Mutex<Option<Waker>>>);
impl Future for SaveWake {
    type Output = BoundaryResult<u32>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        *self.0.lock().unwrap() = Some(cx.waker().clone());
        Poll::Pending
    }
}
#[test]
fn late_cross_thread_wake_does_not_revive_cancelled_generation() {
    let wake = Arc::new(Mutex::new(None));
    let saved = wake.clone();
    let id = spawn_scoped((), move |_| Box::pin(SaveWake(saved))).unwrap();
    poll_task(id).unwrap();
    assert!(cancel_task(id));
    let next = spawn_scoped((), |_| Box::pin(std::future::pending())).unwrap();
    assert_ne!(id, next);
    poll_task(next).unwrap();
    let old = wake.lock().unwrap().take().unwrap();
    std::thread::spawn(move || old.wake()).join().unwrap();
    assert_eq!(__ocamljs_task_ready(next), 0);
    assert!(poll_task(id).is_err());
    assert!(cancel_task(next));
}
struct Counted(Rc<Cell<u32>>);
ocamljs_bindgen::opaque_storage!(Counted);
impl Drop for Counted {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn moves_preflight_aliases_before_consuming_and_drop_exactly_once() {
    unsafe {
        let drops = Rc::new(Cell::new(0));
        let id = insert(Counted(drops.clone()));
        let move_anchor = MoveAnchor::<Counted>::new(id).unwrap();
        assert!(SharedAnchor::<Counted>::new(id).is_err());
        drop(move_anchor);
        assert_eq!(drops.get(), 0);
        assert_eq!(__ocamljs_contains(id), 1);
        let value = MoveAnchor::<Counted>::new(id).unwrap().take();
        assert_eq!(__ocamljs_contains(id), 0);
        drop(value);
        assert_eq!(drops.get(), 1);
    }
}
#[test]
fn view_tokens_pin_parent_and_block_structural_mutation() {
    let mut owner = views::ViewOwner::new(vec![1, 2]);
    let view = owner.view();
    let child = view.child();
    assert!(owner.mutate(|v| v.push(3)).is_err());
    drop(owner);
    assert_eq!(view.read(|v| v[0]), 1);
    drop(view);
    assert_eq!(child.read(|v| v[1]), 2);
    drop(child);
}
#[test]
fn exhausted_task_generations_do_not_revive_stale_ids() {
    let stale = spawn_scoped((), |_| Box::pin(std::future::pending())).unwrap();
    assert!(cancel_task(stale));
    for _ in 0..4200 {
        let id = spawn_scoped((), |_| Box::pin(std::future::pending())).unwrap();
        assert_ne!(id, stale);
        assert!(cancel_task(id));
    }
    assert!(poll_task(stale).is_err());
    assert_eq!(task_count(), 0);
}
