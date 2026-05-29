use crate::pokemon::Pokemon;

pub const EXP_MULTIPLIER: u32 = 20;
pub const MONEY_MULTIPLIER: u32 = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum BattleKind {
    Wild,
    Trainer,
    GymLeader,
}

impl BattleKind {
    fn reward_multiplier(&self) -> u32 {
        match self {
            BattleKind::Wild => 1,
            BattleKind::Trainer => 2,
            BattleKind::GymLeader => 4,
        }
    }
}

#[derive(Debug)]
pub struct BattleReward {
    pub exp: u32,
    pub money: u32,
}

/// Calcola EXP e soldi guadagnati dopo una vittoria.
///
/// exp  = floor(base_exp * enemy_level / 7) * multiplier
/// money = round_to_10(base_exp * enemy_level / 10 * multiplier)
pub fn calculate_reward(enemy: &Pokemon, kind: &BattleKind) -> BattleReward {
    let mult = kind.reward_multiplier();
    let base = enemy.base_experience;
    let level = enemy.level as u32;

    let exp = (base * level / 7) * mult * EXP_MULTIPLIER;
    let money_raw = base * level / 10 * mult * MONEY_MULTIPLIER;
    let money = ((money_raw + 5) / 10) * 10;

    BattleReward { exp, money }
}

/// Distribuisce EXP a tutti i Pokémon non KO nel team.
/// Restituisce il numero totale di level-up avvenuti.
pub fn distribute_exp(team: &mut Vec<Pokemon>, exp: u32) -> u32 {
    let active: Vec<usize> = team.iter().enumerate()
        .filter(|(_, p)| !p.is_fainted())
        .map(|(i, _)| i)
        .collect();

    if active.is_empty() {
        return 0;
    }

    let exp_each = exp / active.len() as u32;
    let mut total_levels = 0u32;
    for idx in active {
        total_levels += team[idx].add_exp(exp_each);
    }
    total_levels
}

/// Livello medio del team (considera solo Pokémon non KO, fallback su tutti).
pub fn team_average_level(team: &[Pokemon]) -> u8 {
    let active: Vec<u8> = team.iter()
        .filter(|p| !p.is_fainted())
        .map(|p| p.level)
        .collect();

    let pool = if active.is_empty() { team.iter().map(|p| p.level).collect::<Vec<_>>() } else { active };
    if pool.is_empty() {
        return 5;
    }
    let sum: u32 = pool.iter().map(|&l| l as u32).sum();
    (sum / pool.len() as u32) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::Stats;
    use crate::types::Type;

    fn enemy(base_exp: u32, level: u8) -> Pokemon {
        Pokemon::new("enemy", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), base_exp, level)
    }

    fn alive(level: u8) -> Pokemon {
        Pokemon::new("alive", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, level)
    }

    #[test]
    fn exp_selvatico() {
        let e = enemy(100, 10);
        let r = calculate_reward(&e, &BattleKind::Wild);
        assert_eq!(r.exp, 100 * 10 / 7 * 1 * EXP_MULTIPLIER);
    }

    #[test]
    fn exp_allenatore_doppia() {
        let e = enemy(100, 10);
        let wild = calculate_reward(&e, &BattleKind::Wild);
        let trainer = calculate_reward(&e, &BattleKind::Trainer);
        assert_eq!(trainer.exp, wild.exp * 2);
    }

    #[test]
    fn exp_capopalestra_quadrupla() {
        let e = enemy(100, 10);
        let wild = calculate_reward(&e, &BattleKind::Wild);
        let gym = calculate_reward(&e, &BattleKind::GymLeader);
        assert_eq!(gym.exp, wild.exp * 4);
    }

    #[test]
    fn money_arrotondato_al_decimo() {
        let e = enemy(100, 10);
        let r = calculate_reward(&e, &BattleKind::Wild);
        assert_eq!(r.money % 10, 0);
    }

    #[test]
    fn distribute_exp_diviso_tra_attivi() {
        let mut team = vec![alive(5), alive(5)];
        distribute_exp(&mut team, 200);
        assert_eq!(team[0].current_exp, 100);
        assert_eq!(team[1].current_exp, 100);
    }

    #[test]
    fn distribute_exp_ignora_ko() {
        let mut team = vec![alive(5), alive(5)];
        team[1].current_hp = 0;
        distribute_exp(&mut team, 100);
        assert_eq!(team[0].current_exp, 100);
        assert_eq!(team[1].current_exp, 0);
    }

    #[test]
    fn team_average_level_corretto() {
        let team = vec![alive(10), alive(20), alive(30)];
        assert_eq!(team_average_level(&team), 20);
    }
}
