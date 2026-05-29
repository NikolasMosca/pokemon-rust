use crate::types::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

/// Effetto aggiuntivo di una mossa oltre al danno diretto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveEffect {
    /// Nessun effetto aggiuntivo (default).
    None,
    /// Guarigione: ripristina `percent`% degli HP massimi del Pokémon che usa la mossa.
    Heal { percent: u8 },
    /// Drain: infligge danno normale e trasferisce al user `percent`% del danno inflitto.
    Drain { percent: u8 },
}

#[derive(Debug, Clone)]
pub struct Move {
    pub name: &'static str,
    pub move_type: Type,
    pub category: MoveCategory,
    pub power: u8,
    pub accuracy: u8,
    pub max_pp: u8,
    pub current_pp: u8,
    pub effect: MoveEffect,
}

impl Move {
    pub const fn new(
        name: &'static str,
        move_type: Type,
        category: MoveCategory,
        power: u8,
        accuracy: u8,
        pp: u8,
    ) -> Self {
        Self { name, move_type, category, power, accuracy, max_pp: pp, current_pp: pp, effect: MoveEffect::None }
    }

    pub const fn with_effect(mut self, effect: MoveEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn has_pp(&self) -> bool {
        self.current_pp > 0
    }

    pub fn use_pp(&mut self) {
        self.current_pp = self.current_pp.saturating_sub(1);
    }

    /// Restituisce true se la mossa infligge danno diretto (power > 0 e non Status).
    pub fn deals_damage(&self) -> bool {
        self.power > 0 && self.category != MoveCategory::Status
    }

    /// Restituisce true se la mossa è supportata dal sistema di gioco:
    /// - Physical/Special con power > 0 (danno diretto), OPPURE
    /// - Mossa con MoveEffect riconosciuto (Heal, Drain).
    /// Le mosse Status senza effetto implementato non sono supportate.
    pub fn is_supported(&self) -> bool {
        match &self.effect {
            MoveEffect::Heal { .. } | MoveEffect::Drain { .. } => true,
            MoveEffect::None => self.deals_damage(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crea_mossa_base() {
        let tackle = Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35);
        assert_eq!(tackle.name, "Tackle");
        assert_eq!(tackle.power, 40);
        assert_eq!(tackle.max_pp, 35);
        assert_eq!(tackle.current_pp, 35);
    }

    #[test]
    fn use_pp_decrementa() {
        let mut m = Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 5);
        m.use_pp();
        assert_eq!(m.current_pp, 4);
        assert!(m.has_pp());
    }

    #[test]
    fn use_pp_non_va_sotto_zero() {
        let mut m = Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 1);
        m.use_pp();
        m.use_pp();
        assert_eq!(m.current_pp, 0);
        assert!(!m.has_pp());
    }

    #[test]
    fn is_supported_mossa_fisica_con_danno() {
        let m = Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35);
        assert!(m.is_supported());
    }

    #[test]
    fn is_supported_mossa_speciale_con_danno() {
        let m = Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15);
        assert!(m.is_supported());
    }

    #[test]
    fn is_supported_status_senza_effetto_non_supportata() {
        let m = Move::new("Growl", Type::Normal, MoveCategory::Status, 0, 100, 40);
        assert!(!m.is_supported());
    }

    #[test]
    fn is_supported_physical_power_zero_non_supportata() {
        // Mossa fisica con power=0 (es. splash) — non fa danno e non ha effetto
        let m = Move::new("Splash", Type::Normal, MoveCategory::Physical, 0, 100, 40);
        assert!(!m.is_supported());
    }

    #[test]
    fn is_supported_heal_supportata_anche_senza_danno() {
        let m = Move::new("Recover", Type::Normal, MoveCategory::Status, 0, 100, 10)
            .with_effect(MoveEffect::Heal { percent: 50 });
        assert!(m.is_supported());
    }

    #[test]
    fn is_supported_drain_supportata() {
        let m = Move::new("Mega Drain", Type::Grass, MoveCategory::Special, 40, 100, 15)
            .with_effect(MoveEffect::Drain { percent: 50 });
        assert!(m.is_supported());
    }
}
