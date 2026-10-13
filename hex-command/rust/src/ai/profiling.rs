//! Opt-in native decision measurements. Disabled runs only test a cached flag.
use super::Policy;
use crate::data::Side;
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
static ENABLED: OnceLock<bool> = OnceLock::new();
const SLOTS: usize = Policy::ALL.len() * 2;
static TIME: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
static CALLS: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
#[repr(C)]
struct Timespec {
    seconds: i64,
    nanos: i64,
}
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
unsafe extern "C" {
    fn clock_gettime(clock: i32, value: *mut Timespec) -> i32;
}
fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var_os("HEX_COMMAND_AI_PROFILE").is_some_and(|value| value == "1")
    })
}
#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
fn now() -> u64 {
    let mut value = Timespec {
        seconds: 0,
        nanos: 0,
    };
    // Linux CLOCK_THREAD_CPUTIME_ID measures actual CPU, excluding descheduling.
    let status = unsafe { clock_gettime(3, &mut value) };
    assert_eq!(status, 0, "thread CPU clock unavailable");
    value.seconds as u64 * 1_000_000_000 + value.nanos as u64
}
#[cfg(not(all(target_os = "linux", target_pointer_width = "64")))]
fn now() -> u64 {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_nanos() as u64
}
pub(super) struct Decision {
    start: u64,
    index: usize,
}
impl Decision {
    #[inline]
    pub fn start(policy: Policy, side: Side) -> Option<Self> {
        if !enabled() {
            return None;
        }
        let policy = Policy::ALL.iter().position(|p| *p == policy).unwrap();
        Some(Self {
            start: now(),
            index: policy * 2 + side.index(),
        })
    }
}
impl Drop for Decision {
    fn drop(&mut self) {
        TIME[self.index].fetch_add(now() - self.start, Ordering::Relaxed);
        CALLS[self.index].fetch_add(1, Ordering::Relaxed);
    }
}
/// Write measurements once, after the native CLI completes (including stdio).
pub fn write() {
    if !enabled() {
        return;
    }
    let mut rows = vec![];
    for (index, policy) in Policy::ALL.iter().enumerate() {
        for side in [Side::Player, Side::Enemy] {
            let slot = index * 2 + side.index();
            let calls = CALLS[slot].load(Ordering::Relaxed);
            if calls > 0 {
                rows.push(serde_json::json!({"policy":policy.as_str(),"side":side,"calls":calls,"seconds":TIME[slot].load(Ordering::Relaxed) as f64/1e9}));
            }
        }
    }
    let clock = if cfg!(all(target_os = "linux", target_pointer_width = "64")) {
        "thread_cpu"
    } else {
        "elapsed"
    };
    eprintln!(
        "AI_PROFILE {}",
        serde_json::json!({"clock":clock,"rows":rows,"instrumentation": "one thread CPU interval per returned decision"})
    );
}
