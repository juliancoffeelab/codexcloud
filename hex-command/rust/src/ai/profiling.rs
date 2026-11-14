//! Opt-in native decision measurements. Disabled runs only test a cached flag.
use super::Policy;
use crate::data::TurnPhase;
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
static ENABLED: OnceLock<bool> = OnceLock::new();
const SLOTS: usize = Policy::ALL.len() * 2;
static TIME: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
static CALLS: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
static TURN_TIME: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
static TURNS: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];
const RECRUITERS: [super::Recruiter; 7] = [
    super::Recruiter::Classic,
    super::Recruiter::Rotation,
    super::Recruiter::Frontline,
    super::Recruiter::Balanced,
    super::Recruiter::Situational,
    super::Recruiter::Predictor,
    super::Recruiter::AggressivePredictor,
];
const RECRUIT_SLOTS: usize = RECRUITERS.len() * 2;
static RECRUIT_TIME: [AtomicU64; RECRUIT_SLOTS] = [const { AtomicU64::new(0) }; RECRUIT_SLOTS];
static RECRUIT_CALLS: [AtomicU64; RECRUIT_SLOTS] = [const { AtomicU64::new(0) }; RECRUIT_SLOTS];

/// Actual recruitment, including execution in the planning draft and attempts
/// after commander movement. These intervals can be nested inside planning.
pub(crate) struct Recruitment {
    start: u64,
    index: usize,
}
impl Recruitment {
    #[inline]
    pub(crate) fn start(kind: super::Recruiter, side: TurnPhase) -> Option<Self> {
        if !enabled() {
            return None;
        }
        let index = RECRUITERS.iter().position(|&r| r == kind).unwrap() * 2 + side.index();
        Some(Self {
            start: now(),
            index,
        })
    }
}
impl Drop for Recruitment {
    fn drop(&mut self) {
        RECRUIT_TIME[self.index].fetch_add(now() - self.start, Ordering::Relaxed);
        RECRUIT_CALLS[self.index].fetch_add(1, Ordering::Relaxed);
    }
}
/// Enable CLI profiling before any worker starts.
pub fn enable() {
    let _ = ENABLED.set(true);
}
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
pub(crate) struct Decision {
    start: u64,
    index: usize,
}
impl Decision {
    #[inline]
    pub fn start(policy: Policy, side: TurnPhase) -> Option<Self> {
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
/// Measures a successful own phase, including decisions, execution and snapshots.
pub(crate) struct Turn {
    start: u64,
    index: usize,
}
impl Turn {
    pub(crate) fn start(policy: Policy, side: TurnPhase) -> Option<Self> {
        if !enabled() {
            return None;
        }
        Some(Self {
            start: now(),
            index: Policy::ALL.iter().position(|p| *p == policy).unwrap() * 2 + side.index(),
        })
    }
    pub(crate) fn finish(self) {
        let elapsed = now() - self.start;
        TURN_TIME[self.index].fetch_add(elapsed, Ordering::Relaxed);
        TURNS[self.index].fetch_add(1, Ordering::Relaxed);
    }
}
fn measurement(policy: Policy, side: Option<TurnPhase>) -> serde_json::Value {
    let index = Policy::ALL.iter().position(|p| *p == policy).unwrap() * 2;
    let slots: Vec<_> = match side {
        Some(side) => vec![index + side.index()],
        None => vec![index, index + 1],
    };
    let sum = |values: &[AtomicU64; SLOTS]| {
        slots
            .iter()
            .map(|&slot| values[slot].load(Ordering::Relaxed))
            .sum::<u64>()
    };
    let calls = sum(&CALLS);
    let decisions = sum(&TIME) as f64;
    let turns = sum(&TURNS);
    let turn_time = sum(&TURN_TIME) as f64;
    let mut row = serde_json::json!({
        "policy":policy.as_str(),"calls":calls,"seconds":decisions/1e9,
        "turns":turns,"turnSeconds":turn_time/1e9,
        "meanDecisionMicrosPerTurn":(turns>0).then(|| decisions/turns as f64/1e3),
        "meanTurnMicros":(turns>0).then(|| turn_time/turns as f64/1e3)
    });
    if let Some(side) = side {
        row["side"] = serde_json::json!(side);
    }
    row
}
/// Per-seat observations and turn-weighted policy totals for a finished run.
pub fn snapshot() -> Option<serde_json::Value> {
    if !enabled() {
        return None;
    }
    let policies: Vec<_> = Policy::ALL
        .into_iter()
        .map(|p| measurement(p, None))
        .filter(|row| row["calls"].as_u64().unwrap() > 0)
        .collect();
    let rows: Vec<_> = Policy::ALL
        .into_iter()
        .flat_map(|p| {
            [TurnPhase::Player, TurnPhase::Ai]
                .into_iter()
                .map(move |s| measurement(p, Some(s)))
        })
        .filter(|row| row["calls"].as_u64().unwrap() > 0)
        .collect();
    let recruiters: Vec<_> = RECRUITERS
        .into_iter()
        .enumerate()
        .flat_map(|(kind, recruiter)| {
            [TurnPhase::Player, TurnPhase::Ai]
                .into_iter()
                .filter_map(move |side| {
                    let index = kind * 2 + side.index();
                    let calls = RECRUIT_CALLS[index].load(Ordering::Relaxed);
                    (calls > 0).then(|| {
                        serde_json::json!({"recruiter":recruiter,"side":side,"calls":calls,
                "seconds":RECRUIT_TIME[index].load(Ordering::Relaxed) as f64 / 1e9})
                    })
                })
        })
        .collect();
    let clock = if cfg!(all(target_os = "linux", target_pointer_width = "64")) {
        "thread_cpu"
    } else {
        "elapsed"
    };
    Some(serde_json::json!({
        "clock":clock,"instrumentation":"one thread CPU interval per returned decision",
        "turnScope":"one side's completed own phase: decisions, command execution and boundary snapshots; excludes turn-transition events, journal encoding and report aggregation",
        "policies":policies,"rows":rows,"recruiters":recruiters
    }))
}
/// Write measurements once, after the native CLI completes (including stdio).
pub fn write() {
    if let Some(profile) = snapshot() {
        eprintln!("AI_PROFILE {profile}");
    }
}
