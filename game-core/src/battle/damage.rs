use crate::battle::rng::Rng;
use crate::moves::{Move, MoveCategory};
use crate::pokemon::Pokemon;
use crate::types::type_effectiveness;

#[derive(Debug, Clone, PartialEq)]
pub struct DamageResult {
    pub damage: u32,
    pub is_crit: bool,
    pub effectiveness: f32,
}

impl DamageResult {
    pub fn zero() -> Self {
        Self { damage: 0, is_crit: false, effectiveness: 1.0 }
    }
}

/// Stat effettivo Gen III+: floor((2*base+31)*level/100) + 5
fn effective_stat(base: u32, level: u8) -> u32 {
    (2 * base + 31) * level as u32 / 100 + 5
}

/// Formula Gen III+: floor((floor(2*L/5+2) * Potere * Atk/Def) / 50 + 2) * STAB * efficacia * crit
/// Crit: ×1.5, probabilità 1/16.
pub fn calculate_damage(attacker: &Pokemon, defender: &Pokemon, move_used: &Move, rng: &mut Rng) -> DamageResult {
    if move_used.power == 0 || matches!(move_used.category, MoveCategory::Status) {
        return DamageResult::zero();
    }

    let (atk, def) = match move_used.category {
        MoveCategory::Physical => (
            effective_stat(attacker.base_stats.attack, attacker.level),
            effective_stat(defender.base_stats.defense, defender.level),
        ),
        MoveCategory::Special => (
            effective_stat(attacker.base_stats.sp_attack, attacker.level),
            effective_stat(defender.base_stats.sp_defense, defender.level),
        ),
        MoveCategory::Status => unreachable!(),
    };

    let level = attacker.level as u32;
    let power = move_used.power as u32;
    let base = (2 * level / 5 + 2) * power * atk / def / 50 + 2;

    let stab = if move_used.move_type == attacker.primary_type
        || attacker.secondary_type == Some(move_used.move_type)
    {
        1.5_f32
    } else {
        1.0_f32
    };

    let effectiveness = type_effectiveness(move_used.move_type, defender.primary_type)
        * defender.secondary_type
            .map(|t| type_effectiveness(move_used.move_type, t))
            .unwrap_or(1.0);

    let is_crit = rng.roll(16);
    let crit = if is_crit { 1.5_f32 } else { 1.0_f32 };

    let damage = (((base as f32) * stab * effectiveness * crit) as u32).max(1);

    DamageResult { damage, is_crit, effectiveness }
}
