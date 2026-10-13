# AI architecture design

## Purpose

Classic is the simple player-facing enemy reference. Simulation players test
three recruitment strategies against the same enemy: infantry only, fair use of
the roster with a stable recruitment pool, and situational unit selection.
Their arena results and campaign completion rates support roster-balance checks.
Campaign players must understand objectives and preserve the actors needed to
complete them. Planning should remain efficient enough for larger maps, armies
and campaigns.

The competence target is a reasonable, repeatable chance of completing ordinary
campaigns, roughly at the level of the existing tactics. Useful local coordination
is sufficient. Optimal army assignments, exhaustive enemy search and guaranteed
survival are not requirements.

Recruitment strategy, campaign reasoning and tactical planning are separate
choices. A tactic fixes their composition for its entire run. Each block can
adapt its behavior to the state, but cannot replace another block or switch its
own implementation.

This document specifies the intended architecture and contracts. The Rust
sketches describe those contracts rather than an already implemented API.

## Responsibilities and boundaries

**Campaigner decides what must happen.** It interprets authored mission goals,
campaign progress, required actors and the strategic situation. Each turn it
produces recruitment requirements, tactical objectives and safety constraints.
It owns duration, counters, stage progression and decisions about when to start,
continue or stop a campaign duty.

**Recruiter decides how to satisfy recruitment requirements.** It selects units
and placements using its fixed recruitment strategy, the available roster,
resources and placement constraints. It does not interpret mission kinds or
derive hidden requirements from campaign data.

**Planner decides how units achieve tactical objectives together.** It binds
objectives to units, allocates cooperating units, selects positions and attacks,
and orders actions. It evaluates the resulting battle state against the supplied
safety constraints. It does not interpret Seal, Vigil, Refuge or other authored
mission kinds.

**The engine remains responsible for mechanics.** Movement, damage, recruitment,
turn transitions, campaign progress and victory are resolved through the same
engine commands and rules used by ordinary play.

Campaigner communicates through explicit orders. Recruiter and Planner must not
recover additional campaign instructions through hidden `Game` helpers.

## Crate and module layout

Keep the implementation in the existing Rust crate. Module boundaries provide
the separation while allowing shared engine queries and inexpensive temporary
battle states.

```text
ai/
  mod.rs                 # public entry points
  tactic.rs              # frozen composition and turn execution
  orders.rs              # typed, serializable contracts
  context.rs             # restricted views and temporary planning state
  presets.rs             # construction of named tactics

  recruiter/
    mod.rs               # Recruiter trait
    infantry.rs
    fair.rs
    situational.rs

  campaigner/
    mod.rs               # Campaigner trait
    arena.rs
    mission.rs           # authored missions -> orders

  planner/
    mod.rs               # Planner trait
    classic.rs
    full.rs
    queued.rs
    assignments.rs       # actor binding and cooperating-unit allocation
    safety.rs            # survival constraint evaluation
    search.rs            # reusable movement/combat search
```

Recruiter module names describe distinct policy contracts. Infantry eligibility,
fair roster coverage and situational selection must have explicit definitions;
they must not be inferred from a tactic's name.

Shared search and safety helpers are implementation tools for the selected
Planner. Calling such a helper does not construct or switch to another Planner.

## Views and execution contexts

`CampaignView` exposes the battle plus authored mission information and campaign
progress. Only Campaigner receives this view.

`BattleView` exposes units, terrain, ownership, resources, action availability,
recruitment options and mechanical movement/combat queries. It does not expose
authored mission goals or campaign duration counters. The underlying engine
references remain private to the context implementation.

`RecruitmentContext` provides a battle view and the ability to emit recruitment
decisions. `TacticalContext` provides a battle view and tactical command emission.
Both operate on one temporary planning state and record accepted decisions.
Command emission validates and applies the command to that temporary state, so
subsequent decisions see its effects.

Views borrow engine data; creating a restricted view does not copy the battle.
Use one draft state per turn and reuse movement/threat workspaces. Cheap candidate
scoring should use queries against that state; additional forks or reversible
command application are reserved for bounded cooperative sequence evaluation.

```rust
pub trait Campaigner {
    fn objectives(&mut self, view: CampaignView<'_>) -> CampaignOrders;
}

pub trait Recruiter {
    fn recruit(
        &mut self,
        context: &mut RecruitmentContext<'_>,
        orders: &RecruitmentOrders,
    ) -> RecruitmentReport;
}

pub trait Planner {
    fn plan(
        &mut self,
        context: &mut TacticalContext<'_>,
        orders: &TacticalOrders,
    ) -> PlanningReport;
}
```

## Order vocabulary

Use typed unit and tile identities and explicit unit selectors. A selector can
describe a unit type, family or role; its meaning belongs to the shared contract,
not to a particular campaign or Planner.

```rust
pub struct CampaignOrders {
    pub recruitment: RecruitmentOrders,
    pub tactics: TacticalOrders,
}

pub struct RecruitmentOrders {
    pub requirements: Vec<RecruitRequirement>,
    pub placement_constraints: Vec<PlacementConstraint>,
}

pub enum RecruitRequirement {
    EnsureUnits {
        selector: UnitSelector,
        count: usize,
    },
}

pub struct TacticalOrders {
    pub battle: BattleObjective,
    pub constraints: Vec<TacticalConstraint>,
    pub objectives: Vec<ObjectiveOrder>,
}

pub struct ObjectiveOrder {
    pub id: ObjectiveId,
    pub priority: ObjectivePriority,
    pub objective: TacticalObjective,
}

pub enum ObjectivePriority {
    Normal,
    Important,
    Critical,
}

pub enum BattleObjective {
    BattleToKill,
}

pub enum TacticalConstraint {
    Survive { actor: UnitId },
}

pub enum TacticalObjective {
    GoTo {
        actor: UnitId,
        destination: TileId,
    },
    Occupy {
        actor: UnitId,
        location: TileId,
    },
    Escort {
        actor: UnitId,
        guards: usize,
        selector: UnitSelector,
    },
    ClearPassage {
        actor: UnitId,
        destination: TileId,
    },
}
```

| Order          | Meaning                                                                      |
| -------------- | ---------------------------------------------------------------------------- |
| `EnsureUnits`  | Obtain enough eligible units to satisfy a recruitment requirement.           |
| `GoTo`         | Move the identified actor to a destination. This is the reach objective.     |
| `Occupy`       | Have the identified actor occupy a location for this turn.                   |
| `Escort`       | Allocate eligible guards to protect the actor and its movement.              |
| `ClearPassage` | Arrange allied movement so the actor can reach the destination.              |
| `Survive`      | Apply the protected-actor safety contract to the actor.                      |
| `BattleToKill` | Supply ordinary combat work for units not otherwise constrained or assigned. |

Movement and occupancy objectives have no campaign duration field. Campaigner
reissues `Occupy` each turn while an authored hold is incomplete, and removes or
changes the order when campaign progress calls for that. Planner handles the
current turn's request without knowing how many campaign turns remain.

`EnsureUnits.count` is the required total of eligible living units, not the
number to recruit this turn. Recruiter computes the deficit from the updated
state, including recruits already issued in this turn. Overlapping selectors
must not cause duplicate recruitment when one unit satisfies both requirements.

Placement constraints protect reserved destinations and required access. A newly
recruited unit must not obstruct a seal, an occupancy target or a passage needed
by an actor this turn. Recruitment cannot assume Planner will later move a unit
that is inactive after recruitment.

## Constraints, priorities and assignments

Safety constraints are feasibility checks. They cannot be outweighed by combat
reward or objective progress. Objectives express the desired work and its
priority; shortfalls must be reported explicitly.

Campaigner supplies stable objective IDs while a duty continues across turns.
Planner handles higher priorities first, with stable IDs breaking ties. Priority
does not override safety. Compatible objectives for an actor are resolved
together; conflicting objectives are deferred with a reason.

An actor can have several compatible orders. For example, `GoTo`, `Survive` and
`Escort` refer to the same adept. Planner must resolve them together, using stable
objective identities and consistent actor assignments.

`Escort` refers to the actor, not just the squares around its final destination.
Planner chooses suitable guards and positions based on the actor's intended
movement and enemy movement-plus-attack possibilities. Escort units may attack
when doing so remains compatible with their protective assignment.

Allocate guards greedily from nearby eligible units, retaining existing
assignments while they remain valid. Use simple suitability and distance scores;
global assignment optimization is unnecessary for the intended competence.

`ClearPassage` is a cooperation request. Planner chooses the passage and arranges
the necessary yielding actions before moving the actor. It must account for
action availability and movement legality rather than merely adding a reward for
standing away from a target.

An allocated guard cannot simultaneously be promised to an incompatible duty.
If two guards were requested but only one eligible guard can be assigned,
Planner records the shortfall. It may still make safe partial progress, but must
not claim that the full escort request was fulfilled.

## Protected-actor survival

`Survive { actor }` puts the actor into a deliberately cautious planning mode.
The protected actor should stay outside enemy movement-plus-attack coverage
where feasible, with allies clearing threats or providing a screen before it
advances.

The survival contract is:

1. Estimate next-enemy-phase exposure after the proposed allied actions using
   cached movement-plus-attack pressure and engine damage queries. Include
   relevant mechanical turn transitions and known reinforcements through engine
   queries. Do not enumerate enemy action permutations. A forecast-safe choice
   is safe according to this bounded model, not a guarantee against every enemy
   sequence; conservative pressure estimates can overstate exposure.
2. Prefer zero-exposure positions and action sequences. Nonlethal residual
   exposure may be evaluated when necessary for progress, but predicted death
   cannot be accepted as a profitable trade.
3. Reject attacks that can cause any retaliation damage to the actor. This
   includes modeled damaging retaliation effects. Even a small, nonlethal amount
   is unacceptable under `Survive`; combat reward or a prospective victory does
   not override this restriction.
4. A zero-retaliation attack still has to pass the next-enemy-phase survival
   check. Killing one threat is insufficient if another can then kill the actor.
5. Consider cooperating allies before concluding that the actor has no safe
   choice. Another unit may remove a blocker or enemy and create a safe retreat
   or advance.
6. If no safe sequence can be found, report an unmet survival constraint. Do not
   silently reinterpret a still-lethal attack as safe because it improves on an
   even worse forecast.

An unmet constraint must not freeze the army. Continue useful allied combat and
route clearing, reevaluate the actor when those actions change its options, and
choose a legal defensive fallback if safety remains unattainable. Prefer retreat
or the least exposed position, including staying put when appropriate. Keep the
constraint marked unmet even if the fallback reduces predicted damage; attacks
with retaliation damage remain forbidden. Distinguish exhausted search budget
from finding no safe candidate within the searched set. Neither proves that no
safe sequence exists globally.

For example, a protected companion may have no individually safe move while an
adjacent enemy remains alive. An infantry unit can kill that enemy first, opening
a safe retreat for the companion. Planner must evaluate that sequence instead
of considering only the companion's isolated attack or movement candidates.

## Turn execution and fixed composition

```rust
pub struct Tactic<R, C, P> {
    recruiter: R,
    campaigner: C,
    planner: P,
}

impl<R: Recruiter, C: Campaigner, P: Planner> Tactic<R, C, P> {
    pub fn run_turn(&mut self, state: &State) -> TurnPlan {
        let orders = self.campaigner.objectives(campaign_view(state));
        let mut draft = PlanningContext::fork(state);

        let recruitment = self.recruiter.recruit(
            &mut draft.recruitment_context(),
            &orders.recruitment,
        );

        let planning = self.planner.plan(
            &mut draft.tactical_context(),
            &orders.tactics,
        );

        draft.finish(recruitment, planning)
    }
}
```

Presets construct a tactic once. The three component identities remain fixed
throughout its run, including campaign chapter transitions. There is no
replacement API, and Campaigner returns orders rather than a Planner selector.

Block-local state may evolve: a fair Recruiter can maintain selection history,
Campaigner can maintain strategic context, and Planner can retain useful caches.
That does not change component identity. Restoring a run must preserve its
composition and required block-local state.

Recruitment decisions update the temporary state before tactical planning. This
accounts for changed gold, occupancy and action availability. The returned plan
contains ordered engine commands; ordinary execution still validates them.

Planner binds actors, allocates cooperating units, evaluates candidate sequences,
checks constraints and emits actions against the updated temporary state.
It can adjust search effort internally in response to complex orders without
switching Planner implementations.

## Adept and seal example

Campaigner recalculates the orders each turn from resolved state:

1. **No eligible adept exists.** Request `EnsureUnits` for the needed family.
   Recruiter selects and places an eligible unit. Actor-specific movement orders
   are issued when they can be bound to an existing actor; no fictitious unit ID
   is used. A new recruit's inactivity is respected.
2. **An adept exists and the route is safe.** Issue `GoTo` for the chosen seal,
   with `Survive` for the actor. The remaining army retains `BattleToKill`.
3. **The adept is endangered near the front.** Add `Escort` requesting two
   eligible infantry guards. Planner allocates guards and coordinates their
   positions and actions with the adept's movement.
4. **The adept can reach the seal but allies obstruct access.** Add
   `ClearPassage`. Planner moves the blockers before moving the adept, while
   retaining its survival constraint.
5. **The seal is complete.** Campaigner removes that objective and derives the
   next turn's orders from the remaining mission requirements.

Campaign-capable Recruiters satisfy explicit mission-required totals first,
then apply their infantry, fair or situational strategy to discretionary
recruitment. This specialist exception is part of each fixed policy's contract;
it does not switch Recruiters. An infantry simulation player may therefore
recruit a mandatory adept, while its discretionary army remains infantry.
Report mandatory recruits separately so balance comparisons retain that meaning.

When resources, roster eligibility or legal placement make a requirement
unavailable, report the deficit explicitly. Do not recruit another specialist
each turn once the required total is met. A strictly infantry-only Recruiter
without specialist exceptions is suitable only for scenarios whose requirements
it can satisfy; its incompatibility must not be mistaken for a tactical failure.

For an arena-only tactic, Campaigner returns `BattleToKill` with no special
campaign recruitment requirements or actor duties. It does not select a
different set of blocks during play.

## Results and simulation reporting

`TurnPlan` returns the existing typed engine commands together with recruitment
and planning reports. Reports include:

- the orders supplied by Campaigner;
- actor bindings and guard assignments;
- fulfilled, deferred and unmet requirements;
- explicit reasons such as insufficient resources, unavailable unit types,
  inactive actors, insufficient eligible guards, blocked access or no safe
  sequence;
- safety assessments needed to explain protected-actor decisions.

Keep routine reports compact: order IDs, assignments, status/reason codes and
summary safety estimates are enough. Detailed candidate and safety traces are
opt-in diagnostics. Serialization belongs in the reporting layer, outside
candidate scoring; routine runs must still retain the information the analyzer
needs to explain outcomes.

Simulation records should retain these diagnostics so the maintained analyzer
can explain what was requested, how it was assigned and why it could not be
fulfilled. An outcome report must distinguish an actor death, unfulfilled
recruitment, lack of access, insufficient protection and a round cap.

## Extensibility and performance

New order variants require exhaustive handling. Every consumer must implement
the order or explicitly report unsupported behavior. Filters use nested
exhaustive match functions; enum wildcards and `matches!` cannot silently discard
a new requirement.

Keep selectors, actor binding, assignment, safety evaluation and mechanical
queries reusable across planners. Cache reusable terrain and movement data and
reuse planning buffers. Reevaluate state-dependent assignments and safety after
actions invalidate them. Shared helpers must not introduce hidden campaign
interpretation or mutable component selection.

Cooperative search should focus on the actors and allies involved in a request.
Start with one protected actor and at most two nearby helpers, evaluating at most
two helper actions before the actor acts. Start with a fixed budget of 32–64
candidate states per actor and a fixed total cooperative budget per turn, so many
protected actors cannot cause unbounded work. These are starting limits to tune
through measurement, not demonstrated optimal values. Ordinary combat stays on
the selected Planner's existing heuristic path. Reaching a search limit must be
reported distinctly from finding no safe candidate in the searched set.

Validate implementations with full simulations and the maintained analyzer,
comparing campaign completion and chooser/phase CPU against the existing tactics
under the same configuration. Include the arena balance checks. Aim for comparable
campaign competence and inexpensive local rescues; broaden search only when
measured results justify the extra cost.
