use crate::moves::Move;
use crate::types::Type;

#[derive(Debug, Clone)]
pub struct Stats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub sp_attack: u32,
    pub sp_defense: u32,
    pub speed: u32,
}

impl Stats {
    pub const fn new(hp: u32, attack: u32, defense: u32, sp_attack: u32, sp_defense: u32, speed: u32) -> Self {
        Self { hp, attack, defense, sp_attack, sp_defense, speed }
    }
}

#[derive(Debug, Clone)]
pub struct Pokemon {
    pub name: String,
    pub primary_type: Type,
    pub secondary_type: Option<Type>,
    pub base_stats: Stats,
    pub base_experience: u32,
    pub moves: Vec<Move>,
    pub current_hp: u32,
    pub level: u8,
    pub current_exp: u32,
    /// ID numerico del Pokédex (usato per i cry). None per Pokémon generati senza ID.
    pub pokedex_id: Option<u32>,
}

impl Pokemon {
    pub fn new(
        name: impl Into<String>,
        primary_type: Type,
        secondary_type: Option<Type>,
        base_stats: Stats,
        base_experience: u32,
        level: u8,
    ) -> Self {
        let max_hp = Self::calculate_hp(&base_stats, level);
        Self {
            name: name.into(),
            primary_type,
            secondary_type,
            base_stats,
            base_experience,
            moves: Vec::new(),
            current_hp: max_hp,
            level,
            current_exp: 0,
            pokedex_id: None,
        }
    }

    pub fn max_hp(&self) -> u32 {
        Self::calculate_hp(&self.base_stats, self.level)
    }

    /// Formula HP Gen III+: floor((2 * base + 31) * level / 100) + level + 10
    fn calculate_hp(stats: &Stats, level: u8) -> u32 {
        let level = level as u32;
        (2 * stats.hp + 31) * level / 100 + level + 10
    }

    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }

    pub fn add_move(&mut self, m: Move) {
        if self.moves.len() < 4 {
            self.moves.push(m);
        }
    }

    /// Garantisce che il Pokémon abbia almeno una mossa che infligge danno.
    /// Se tutte le mosse sono Status (o non ce ne sono), aggiunge Tackle come fallback.
    /// Non aggiunge nulla se esiste già almeno una mossa con deals_damage() == true.
    pub fn ensure_damage_move(&mut self) {
        if !self.moves.iter().any(|m| m.deals_damage()) {
            self.add_move(Move::new("tackle", crate::types::Type::Normal, crate::moves::MoveCategory::Physical, 40, 100, 35));
        }
    }

    pub fn take_damage(&mut self, damage: u32) {
        self.current_hp = self.current_hp.saturating_sub(damage);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp());
    }

    pub fn full_heal(&mut self) {
        self.current_hp = self.max_hp();
        for m in &mut self.moves {
            m.current_pp = m.max_pp;
        }
    }

    /// Aggiunge EXP e restituisce quante volte il Pokémon è salito di livello.
    pub fn add_exp(&mut self, exp: u32) -> u32 {
        if self.level >= 100 {
            return 0;
        }
        self.current_exp += exp;
        let mut levels_gained = 0u32;
        while self.level < 100 && self.current_exp >= exp_threshold(self.level + 1) {
            self.current_exp -= exp_threshold(self.level + 1);
            self.level += 1;
            levels_gained += 1;
            let new_max = self.max_hp();
            if self.current_hp > 0 {
                self.current_hp = new_max;
                for m in &mut self.moves {
                    m.current_pp = m.max_pp;
                }
            }
        }
        levels_gained
    }
}

/// Curva "medium fast": soglia EXP per raggiungere `level`.
/// Formula: floor(4 * level^3 / 5)
pub fn exp_threshold(level: u8) -> u32 {
    let l = level as u32;
    4 * l * l * l / 5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{Move, MoveCategory};

    fn bulbasaur() -> Pokemon {
        Pokemon::new("Bulbasaur", Type::Grass, Some(Type::Poison), Stats::new(45, 49, 49, 65, 65, 45), 64, 5)
    }

    #[test]
    fn hp_calcolati_correttamente_livello_5() {
        let p = bulbasaur();
        assert_eq!(p.max_hp(), 21);
        assert_eq!(p.current_hp, 21);
    }

    #[test]
    fn danno_riduce_hp() {
        let mut p = bulbasaur();
        p.take_damage(10);
        assert_eq!(p.current_hp, 11);
    }

    #[test]
    fn danno_eccessivo_porta_a_zero() {
        let mut p = bulbasaur();
        p.take_damage(999);
        assert_eq!(p.current_hp, 0);
        assert!(p.is_fainted());
    }

    #[test]
    fn max_quattro_mosse() {
        let mut p = bulbasaur();
        for i in 0..5 {
            let name = match i { 0 => "A", 1 => "B", 2 => "C", 3 => "D", _ => "E" };
            p.add_move(Move::new(name, Type::Normal, MoveCategory::Physical, 40, 100, 35));
        }
        assert_eq!(p.moves.len(), 4);
    }

    #[test]
    fn level_up_ripristina_pp() {
        use crate::moves::{Move, MoveCategory};
        let mut p = bulbasaur(); // livello 5
        p.add_move(Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35));
        p.moves[0].current_pp = 2;
        p.add_exp(200); // sale a livello 6
        assert_eq!(p.moves[0].current_pp, p.moves[0].max_pp);
    }

    #[test]
    fn add_exp_sale_di_livello() {
        let mut p = bulbasaur(); // livello 5
        // soglia per livello 6 = 4*6^3/5 = 172
        let levels = p.add_exp(200);
        assert_eq!(levels, 1);
        assert_eq!(p.level, 6);
    }

    #[test]
    fn add_exp_piu_livelli_in_una_volta() {
        let mut p = bulbasaur(); // livello 5
        let levels = p.add_exp(10000);
        assert!(levels > 1);
        assert!(p.level > 6);
    }

    #[test]
    fn level_cap_a_100() {
        let mut p = Pokemon::new("Test", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, 99);
        p.add_exp(1_000_000);
        assert_eq!(p.level, 100);
    }

    #[test]
    fn exp_threshold_livello_6() {
        assert_eq!(exp_threshold(6), 172);
    }
}
