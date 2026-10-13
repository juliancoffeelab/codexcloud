use crate::{data::*, logic::Game};
use std::cmp::Ordering;
/// Compare canonical actor identities without formatting in candidate loops.
pub(super) fn actor_matches(actor: &Option<String>, id: u32) -> bool {
    actor.as_deref().is_some_and(|actor| {
        actor.strip_prefix('u').is_some_and(|number| {
            !number.is_empty()
                && (number.len() == 1 || !number.starts_with('0'))
                && number.bytes().all(|byte| byte.is_ascii_digit())
                && number.parse::<u32>() == Ok(id)
        })
    })
}
#[cfg(test)]
mod actor_tests {
    use super::actor_matches;
    #[test]
    fn only_canonical_unit_keys_match() {
        for id in [0, 12, u32::MAX] {
            assert!(actor_matches(&Some(format!("u{id}")), id));
            assert!(!actor_matches(&Some(format!("u0{id}")), id));
        }
        for key in ["", "u", "u+12", "u１２", "u4294967296", "12"] {
            assert!(!actor_matches(&Some(key.into()), 12));
        }
        assert!(!actor_matches(&None, 0));
    }
}
pub fn objective(id: &str, kind: ObjectiveKind, tiles: Vec<String>, weight: f64) -> Objective {
    Objective {
        id: id.into(),
        kind,
        tiles,
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}
pub fn individual(goals: Vec<Objective>) -> Vec<Objective> {
    goals
        .into_iter()
        .flat_map(|g| {
            if g.kind == ObjectiveKind::Capture {
                g.tiles
                    .iter()
                    .map(|t| {
                        let mut o = g.clone();
                        o.id = format!("{}:{t}", g.id);
                        o.tiles = vec![t.clone()];
                        o
                    })
                    .collect()
            } else {
                vec![g]
            }
        })
        .collect()
}
pub(crate) struct TileOrder {
    home: (f64, f64),
    direction: (f64, f64),
    villages: bool,
}
pub(crate) struct TileOrderContext {
    home: Option<(f64, f64)>,
    target: (f64, f64),
    villages: bool,
}
impl TileOrderContext {
    pub(crate) fn for_unit(&self, g: &Game, u: &Unit) -> TileOrder {
        let (c, r) = g.board.positions[u.tile];
        let position = ((c - r.div_euclid(2)) as f64, r as f64);
        let home = self.home.unwrap_or(position);
        let mut direction = (self.target.0 - home.0, self.target.1 - home.1);
        if direction == (0.0, 0.0) {
            direction = (position.0 - home.0, position.1 - home.1);
        }
        if direction == (0.0, 0.0) {
            direction.0 = if u.side == Side::Player { 1.0 } else { -1.0 };
        }
        TileOrder {
            home,
            direction,
            villages: self.villages,
        }
    }
}
impl TileOrder {
    pub(crate) fn context(g: &Game, side: Side, villages: bool) -> TileOrderContext {
        fn center(g: &Game, tiles: impl Iterator<Item = usize>) -> (f64, f64) {
            let (sum, count) = tiles.fold(((0.0, 0.0), 0usize), |(s, n), t| {
                let (c, r) = g.board.positions[t];
                ((s.0 + (c - r.div_euclid(2)) as f64, s.1 + r as f64), n + 1)
            });
            (sum.0 / count as f64, sum.1 / count as f64)
        }
        let mut home_keeps = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, Terrain::Keep) && g.owners[t] == Some(side))
            .peekable();
        let home = if home_keeps.peek().is_some() {
            Some(center(g, home_keeps))
        } else {
            None
        };
        let mut opposing = (0..g.terrain.len())
            .filter(|&t| g.terrain_is(t, Terrain::Keep) && g.owners[t].is_some_and(|s| s != side))
            .peekable();
        let mut enemies = g
            .units
            .iter()
            .filter(|o| o.side != side)
            .map(|o| o.tile)
            .peekable();
        let target = if opposing.peek().is_some() {
            center(g, opposing)
        } else if enemies.peek().is_some() {
            center(g, enemies)
        } else {
            center(g, 0..g.terrain.len())
        };
        TileOrderContext {
            home,
            target,
            villages,
        }
    }
    pub(crate) fn new(g: &Game, u: &Unit, villages: bool) -> Self {
        Self::context(g, u.side, villages).for_unit(g, u)
    }
    pub(crate) fn compare(&self, g: &Game, first: usize, second: usize) -> Ordering {
        let axial = |t: usize| {
            let (c, r) = g.board.positions[t];
            ((c - r.div_euclid(2)) as f64, r as f64)
        };
        let a = axial(first);
        let b = axial(second);
        let av = self.villages && g.terrain_is(first, Terrain::Village);
        let bv = self.villages && g.terrain_is(second, Terrain::Village);
        let home_distance = |p: (f64, f64)| {
            ((p.0 - self.home.0).abs()
                + (p.1 - self.home.1).abs()
                + (p.0 + p.1 - self.home.0 - self.home.1).abs())
                / 2.0
        };
        let values = [
            (bv as i32 - av as i32) as f64,
            if av && bv {
                home_distance(a) - home_distance(b)
            } else {
                0.0
            },
            (b.0 - a.0) * self.direction.0 + (b.1 - a.1) * self.direction.1,
            (a.0 - b.0) * self.direction.1 - (a.1 - b.1) * self.direction.0,
        ];
        values
            .into_iter()
            .find(|v| *v != 0.0)
            .unwrap_or(0.0)
            .partial_cmp(&0.0)
            .unwrap_or(Ordering::Equal)
    }
}
