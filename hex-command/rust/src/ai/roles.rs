use super::{RecruitRole, TacticalRole};
use crate::data::Category;
use crate::logic::Game;
pub(super) fn role(g: &Game<'_>, kind: usize) -> TacticalRole {
    let rule = &g.rules.units[kind];
    if rule.healing > 0 {
        TacticalRole::Healer
    } else if rule.range > 1 {
        TacticalRole::Ranged
    } else {
        TacticalRole::Frontline
    }
}
pub(super) fn recruit_role(g: &Game<'_>, kind: usize) -> RecruitRole {
    let rule = &g.rules.units[kind];
    if rule.healing > 0 {
        RecruitRole::Healer
    } else if rule.r#move >= 5 && rule.attacks[0].category == Category::Melee {
        RecruitRole::Scout
    } else if rule.range > 1 {
        RecruitRole::Ranged
    } else {
        RecruitRole::Frontline
    }
}
