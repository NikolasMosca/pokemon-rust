use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Normal,
    Fire,
    Water,
    Grass,
    Electric,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Moltiplicatore danno attaccante -> difensore.
/// I valori sono clamped a [0.2, 2.0]: nessuna immunità completa (0.0 → 0.2),
/// nessun super-efficace oltre il doppio (2.0 max).
pub fn type_effectiveness(attacker: Type, defender: Type) -> f32 {
    use Type::*;
    match (attacker, defender) {
        // Fire
        (Fire, Grass) | (Fire, Ice) | (Fire, Bug) | (Fire, Steel) => 2.0,
        (Fire, Fire) | (Fire, Water) | (Fire, Rock) | (Fire, Dragon) => 0.5,
        // Water
        (Water, Fire) | (Water, Ground) | (Water, Rock) => 2.0,
        (Water, Water) | (Water, Grass) | (Water, Dragon) => 0.5,
        // Grass
        (Grass, Water) | (Grass, Ground) | (Grass, Rock) => 2.0,
        (Grass, Fire) | (Grass, Grass) | (Grass, Poison) | (Grass, Flying)
        | (Grass, Bug) | (Grass, Dragon) | (Grass, Steel) => 0.5,
        // Electric
        (Electric, Water) | (Electric, Flying) => 2.0,
        (Electric, Grass) | (Electric, Electric) | (Electric, Dragon) | (Electric, Steel) => 0.5,
        (Electric, Ground) => 0.2,
        // Ice
        (Ice, Grass) | (Ice, Ground) | (Ice, Flying) | (Ice, Dragon) => 2.0,
        (Ice, Fire) | (Ice, Water) | (Ice, Ice) | (Ice, Steel) => 0.5,
        // Fighting
        (Fighting, Normal) | (Fighting, Ice) | (Fighting, Rock)
        | (Fighting, Dark) | (Fighting, Steel) => 2.0,
        (Fighting, Poison) | (Fighting, Bug) | (Fighting, Psychic)
        | (Fighting, Flying) | (Fighting, Fairy) => 0.5,
        (Fighting, Ghost) => 0.2,
        // Poison
        (Poison, Grass) | (Poison, Fairy) => 2.0,
        (Poison, Poison) | (Poison, Ground) | (Poison, Rock) | (Poison, Ghost) => 0.5,
        (Poison, Steel) => 0.2,
        // Ground
        (Ground, Fire) | (Ground, Electric) | (Ground, Poison)
        | (Ground, Rock) | (Ground, Steel) => 2.0,
        (Ground, Grass) | (Ground, Bug) => 0.5,
        (Ground, Flying) => 0.2,
        // Flying
        (Flying, Grass) | (Flying, Fighting) | (Flying, Bug) => 2.0,
        (Flying, Electric) | (Flying, Rock) | (Flying, Steel) => 0.5,
        // Psychic
        (Psychic, Fighting) | (Psychic, Poison) => 2.0,
        (Psychic, Psychic) | (Psychic, Steel) => 0.5,
        (Psychic, Dark) => 0.2,
        // Bug
        (Bug, Grass) | (Bug, Psychic) | (Bug, Dark) => 2.0,
        (Bug, Fire) | (Bug, Fighting) | (Bug, Flying) | (Bug, Ghost)
        | (Bug, Steel) | (Bug, Fairy) => 0.5,
        // Rock
        (Rock, Fire) | (Rock, Ice) | (Rock, Flying) | (Rock, Bug) => 2.0,
        (Rock, Fighting) | (Rock, Ground) | (Rock, Steel) => 0.5,
        // Ghost
        (Ghost, Psychic) | (Ghost, Ghost) => 2.0,
        (Ghost, Dark) => 0.5,
        (Ghost, Normal) | (Ghost, Fighting) => 0.2,
        // Dragon
        (Dragon, Dragon) => 2.0,
        (Dragon, Steel) => 0.5,
        (Dragon, Fairy) => 0.2,
        // Dark
        (Dark, Psychic) | (Dark, Ghost) => 2.0,
        (Dark, Fighting) | (Dark, Dark) | (Dark, Fairy) => 0.5,
        // Steel
        (Steel, Ice) | (Steel, Rock) | (Steel, Fairy) => 2.0,
        (Steel, Fire) | (Steel, Water) | (Steel, Electric) | (Steel, Steel) => 0.5,
        // Fairy
        (Fairy, Fighting) | (Fairy, Dragon) | (Fairy, Dark) => 2.0,
        (Fairy, Fire) | (Fairy, Poison) | (Fairy, Steel) => 0.5,
        // Normal
        (Normal, Rock) | (Normal, Steel) => 0.5,
        (Normal, Ghost) => 0.2,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuoco_super_efficace_su_erba() {
        assert_eq!(type_effectiveness(Type::Fire, Type::Grass), 2.0);
    }

    #[test]
    fn acqua_non_molto_efficace_su_drago() {
        assert_eq!(type_effectiveness(Type::Water, Type::Dragon), 0.5);
    }

    #[test]
    fn elettro_immune_su_terra() {
        assert_eq!(type_effectiveness(Type::Electric, Type::Ground), 0.2);
    }

    #[test]
    fn normale_contro_normale_neutro() {
        assert_eq!(type_effectiveness(Type::Normal, Type::Normal), 1.0);
    }

    #[test]
    fn spettro_ridotto_contro_normale() {
        assert_eq!(type_effectiveness(Type::Ghost, Type::Normal), 0.2);
    }

    #[test]
    fn spettro_ridotto_contro_lotta() {
        assert_eq!(type_effectiveness(Type::Ghost, Type::Fighting), 0.2);
    }

    #[test]
    fn drago_ridotto_a_folletto() {
        assert_eq!(type_effectiveness(Type::Dragon, Type::Fairy), 0.2);
    }

    #[test]
    fn veleno_ridotto_su_acciaio() {
        assert_eq!(type_effectiveness(Type::Poison, Type::Steel), 0.2);
    }

    #[test]
    fn psichico_ridotto_su_buio() {
        assert_eq!(type_effectiveness(Type::Psychic, Type::Dark), 0.2);
    }

    #[test]
    fn normale_ridotto_su_spettro() {
        assert_eq!(type_effectiveness(Type::Normal, Type::Ghost), 0.2);
    }

    #[test]
    fn terra_ridotta_su_volante() {
        assert_eq!(type_effectiveness(Type::Ground, Type::Flying), 0.2);
    }

    #[test]
    fn lotta_ridotta_su_spettro() {
        assert_eq!(type_effectiveness(Type::Fighting, Type::Ghost), 0.2);
    }

    #[test]
    fn elettro_non_molto_efficace_su_acciaio() {
        assert_eq!(type_effectiveness(Type::Electric, Type::Steel), 0.5);
    }

    #[test]
    fn ghiaccio_non_molto_efficace_su_acciaio() {
        assert_eq!(type_effectiveness(Type::Ice, Type::Steel), 0.5);
    }

    #[test]
    fn folletto_non_molto_efficace_su_veleno() {
        assert_eq!(type_effectiveness(Type::Fairy, Type::Poison), 0.5);
    }

    #[test]
    fn simmetria_veleno_folletto() {
        assert_eq!(type_effectiveness(Type::Poison, Type::Fairy), 2.0);
        assert_eq!(type_effectiveness(Type::Fairy, Type::Poison), 0.5);
    }

    #[test]
    fn fuoco_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Fire, Type::Ice), 2.0);
        assert_eq!(type_effectiveness(Type::Fire, Type::Bug), 2.0);
        assert_eq!(type_effectiveness(Type::Fire, Type::Steel), 2.0);
        assert_eq!(type_effectiveness(Type::Fire, Type::Water), 0.5);
        assert_eq!(type_effectiveness(Type::Fire, Type::Rock), 0.5);
        assert_eq!(type_effectiveness(Type::Fire, Type::Dragon), 0.5);
        assert_eq!(type_effectiveness(Type::Fire, Type::Fire), 0.5);
    }

    #[test]
    fn acqua_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Water, Type::Ground), 2.0);
        assert_eq!(type_effectiveness(Type::Water, Type::Rock), 2.0);
        assert_eq!(type_effectiveness(Type::Water, Type::Water), 0.5);
        assert_eq!(type_effectiveness(Type::Water, Type::Grass), 0.5);
    }

    #[test]
    fn erba_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Grass, Type::Water), 2.0);
        assert_eq!(type_effectiveness(Type::Grass, Type::Ground), 2.0);
        assert_eq!(type_effectiveness(Type::Grass, Type::Rock), 2.0);
        assert_eq!(type_effectiveness(Type::Grass, Type::Fire), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Grass), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Poison), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Flying), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Bug), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Dragon), 0.5);
        assert_eq!(type_effectiveness(Type::Grass, Type::Steel), 0.5);
    }

    #[test]
    fn elettro_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Electric, Type::Water), 2.0);
        assert_eq!(type_effectiveness(Type::Electric, Type::Flying), 2.0);
        assert_eq!(type_effectiveness(Type::Electric, Type::Grass), 0.5);
        assert_eq!(type_effectiveness(Type::Electric, Type::Electric), 0.5);
        assert_eq!(type_effectiveness(Type::Electric, Type::Dragon), 0.5);
    }

    #[test]
    fn ghiaccio_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Ice, Type::Grass), 2.0);
        assert_eq!(type_effectiveness(Type::Ice, Type::Ground), 2.0);
        assert_eq!(type_effectiveness(Type::Ice, Type::Flying), 2.0);
        assert_eq!(type_effectiveness(Type::Ice, Type::Dragon), 2.0);
        assert_eq!(type_effectiveness(Type::Ice, Type::Fire), 0.5);
        assert_eq!(type_effectiveness(Type::Ice, Type::Water), 0.5);
        assert_eq!(type_effectiveness(Type::Ice, Type::Ice), 0.5);
    }

    #[test]
    fn lotta_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Fighting, Type::Normal), 2.0);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Ice), 2.0);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Rock), 2.0);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Dark), 2.0);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Steel), 2.0);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Poison), 0.5);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Bug), 0.5);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Psychic), 0.5);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Flying), 0.5);
        assert_eq!(type_effectiveness(Type::Fighting, Type::Fairy), 0.5);
    }

    #[test]
    fn veleno_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Poison, Type::Grass), 2.0);
        assert_eq!(type_effectiveness(Type::Poison, Type::Poison), 0.5);
        assert_eq!(type_effectiveness(Type::Poison, Type::Ground), 0.5);
        assert_eq!(type_effectiveness(Type::Poison, Type::Rock), 0.5);
        assert_eq!(type_effectiveness(Type::Poison, Type::Ghost), 0.5);
    }

    #[test]
    fn terra_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Ground, Type::Fire), 2.0);
        assert_eq!(type_effectiveness(Type::Ground, Type::Electric), 2.0);
        assert_eq!(type_effectiveness(Type::Ground, Type::Poison), 2.0);
        assert_eq!(type_effectiveness(Type::Ground, Type::Rock), 2.0);
        assert_eq!(type_effectiveness(Type::Ground, Type::Steel), 2.0);
        assert_eq!(type_effectiveness(Type::Ground, Type::Grass), 0.5);
        assert_eq!(type_effectiveness(Type::Ground, Type::Bug), 0.5);
    }

    #[test]
    fn volante_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Flying, Type::Grass), 2.0);
        assert_eq!(type_effectiveness(Type::Flying, Type::Fighting), 2.0);
        assert_eq!(type_effectiveness(Type::Flying, Type::Bug), 2.0);
        assert_eq!(type_effectiveness(Type::Flying, Type::Electric), 0.5);
        assert_eq!(type_effectiveness(Type::Flying, Type::Rock), 0.5);
        assert_eq!(type_effectiveness(Type::Flying, Type::Steel), 0.5);
    }

    #[test]
    fn psichico_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Psychic, Type::Fighting), 2.0);
        assert_eq!(type_effectiveness(Type::Psychic, Type::Poison), 2.0);
        assert_eq!(type_effectiveness(Type::Psychic, Type::Psychic), 0.5);
        assert_eq!(type_effectiveness(Type::Psychic, Type::Steel), 0.5);
    }

    #[test]
    fn coleottero_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Bug, Type::Grass), 2.0);
        assert_eq!(type_effectiveness(Type::Bug, Type::Psychic), 2.0);
        assert_eq!(type_effectiveness(Type::Bug, Type::Dark), 2.0);
        assert_eq!(type_effectiveness(Type::Bug, Type::Fire), 0.5);
        assert_eq!(type_effectiveness(Type::Bug, Type::Fighting), 0.5);
        assert_eq!(type_effectiveness(Type::Bug, Type::Flying), 0.5);
        assert_eq!(type_effectiveness(Type::Bug, Type::Ghost), 0.5);
        assert_eq!(type_effectiveness(Type::Bug, Type::Steel), 0.5);
        assert_eq!(type_effectiveness(Type::Bug, Type::Fairy), 0.5);
    }

    #[test]
    fn roccia_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Rock, Type::Fire), 2.0);
        assert_eq!(type_effectiveness(Type::Rock, Type::Ice), 2.0);
        assert_eq!(type_effectiveness(Type::Rock, Type::Flying), 2.0);
        assert_eq!(type_effectiveness(Type::Rock, Type::Bug), 2.0);
        assert_eq!(type_effectiveness(Type::Rock, Type::Fighting), 0.5);
        assert_eq!(type_effectiveness(Type::Rock, Type::Ground), 0.5);
        assert_eq!(type_effectiveness(Type::Rock, Type::Steel), 0.5);
    }

    #[test]
    fn spettro_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Ghost, Type::Psychic), 2.0);
        assert_eq!(type_effectiveness(Type::Ghost, Type::Ghost), 2.0);
        assert_eq!(type_effectiveness(Type::Ghost, Type::Dark), 0.5);
    }

    #[test]
    fn drago_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Dragon, Type::Dragon), 2.0);
        assert_eq!(type_effectiveness(Type::Dragon, Type::Steel), 0.5);
    }

    #[test]
    fn buio_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Dark, Type::Psychic), 2.0);
        assert_eq!(type_effectiveness(Type::Dark, Type::Ghost), 2.0);
        assert_eq!(type_effectiveness(Type::Dark, Type::Fighting), 0.5);
        assert_eq!(type_effectiveness(Type::Dark, Type::Dark), 0.5);
        assert_eq!(type_effectiveness(Type::Dark, Type::Fairy), 0.5);
    }

    #[test]
    fn acciaio_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Steel, Type::Ice), 2.0);
        assert_eq!(type_effectiveness(Type::Steel, Type::Rock), 2.0);
        assert_eq!(type_effectiveness(Type::Steel, Type::Fairy), 2.0);
        assert_eq!(type_effectiveness(Type::Steel, Type::Fire), 0.5);
        assert_eq!(type_effectiveness(Type::Steel, Type::Water), 0.5);
        assert_eq!(type_effectiveness(Type::Steel, Type::Electric), 0.5);
        assert_eq!(type_effectiveness(Type::Steel, Type::Steel), 0.5);
    }

    #[test]
    fn folletto_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Fairy, Type::Fighting), 2.0);
        assert_eq!(type_effectiveness(Type::Fairy, Type::Dragon), 2.0);
        assert_eq!(type_effectiveness(Type::Fairy, Type::Dark), 2.0);
        assert_eq!(type_effectiveness(Type::Fairy, Type::Fire), 0.5);
        assert_eq!(type_effectiveness(Type::Fairy, Type::Steel), 0.5);
    }

    #[test]
    fn normale_tutte_le_interazioni() {
        assert_eq!(type_effectiveness(Type::Normal, Type::Rock), 0.5);
        assert_eq!(type_effectiveness(Type::Normal, Type::Steel), 0.5);
    }
}
