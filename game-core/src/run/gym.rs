pub const TOTAL_GYMS: u8 = 8;
pub const TRAINERS_PER_GYM: u8 = 3;
pub const WILDS_PER_TRAINER: u8 = 3;
pub const POKECENTER_MAX_PER_GYM: u8 = 3;

pub use crate::run::rewards::BattleKind;

#[derive(Debug, Clone)]
pub struct GymProgress {
    pub gym_index: u8,
    pub trainers_defeated: u8,
    pub wilds_since_last_trainer: u8,
    pub pokecenter_uses: u8,
}

#[derive(Debug, PartialEq)]
pub enum NextOpponent {
    Wild,
    Trainer,
    GymLeader,
}

impl From<NextOpponent> for BattleKind {
    fn from(n: NextOpponent) -> Self {
        match n {
            NextOpponent::Wild => BattleKind::Wild,
            NextOpponent::Trainer => BattleKind::Trainer,
            NextOpponent::GymLeader => BattleKind::GymLeader,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GymError {
    PokecenterLimitReached,
    AlreadyComplete,
}

impl GymProgress {
    pub fn new() -> Self {
        Self {
            gym_index: 0,
            trainers_defeated: 0,
            wilds_since_last_trainer: 0,
            pokecenter_uses: 0,
        }
    }

    /// Schema per palestra: 3 wild → trainer → 3 wild → trainer → 3 wild → capopalestra.
    pub fn next_opponent(&self) -> NextOpponent {
        if self.trainers_defeated >= TRAINERS_PER_GYM {
            return NextOpponent::GymLeader;
        }
        if self.wilds_since_last_trainer < WILDS_PER_TRAINER {
            NextOpponent::Wild
        } else {
            NextOpponent::Trainer
        }
    }

    pub fn record_wild_defeated(&mut self) {
        if self.wilds_since_last_trainer < WILDS_PER_TRAINER {
            self.wilds_since_last_trainer += 1;
        }
    }

    pub fn can_use_pokecenter(&self) -> bool {
        self.pokecenter_uses < POKECENTER_MAX_PER_GYM
    }

    pub fn use_pokecenter(&mut self) -> Result<(), GymError> {
        if !self.can_use_pokecenter() {
            return Err(GymError::PokecenterLimitReached);
        }
        self.pokecenter_uses += 1;
        Ok(())
    }

    pub fn record_trainer_defeated(&mut self) {
        if self.trainers_defeated < TRAINERS_PER_GYM {
            self.trainers_defeated += 1;
            self.wilds_since_last_trainer = 0;
        }
    }

    /// Avanza alla palestra successiva. Restituisce Err se tutte le palestre
    /// sono state completate (fine run).
    pub fn advance(&mut self) -> Result<(), GymError> {
        if self.gym_index >= TOTAL_GYMS - 1 {
            return Err(GymError::AlreadyComplete);
        }
        self.gym_index += 1;
        self.trainers_defeated = 0;
        self.wilds_since_last_trainer = 0;
        self.pokecenter_uses = 0;
        Ok(())
    }

    pub fn is_run_complete(&self) -> bool {
        self.gym_index >= TOTAL_GYMS
    }

    pub fn badges(&self) -> u8 {
        self.gym_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inizia_con_wild() {
        let g = GymProgress::new();
        assert_eq!(g.next_opponent(), NextOpponent::Wild);
    }

    #[test]
    fn dopo_3_wild_tocca_al_trainer() {
        let mut g = GymProgress::new();
        g.record_wild_defeated();
        g.record_wild_defeated();
        g.record_wild_defeated();
        assert_eq!(g.next_opponent(), NextOpponent::Trainer);
    }

    #[test]
    fn dopo_trainer_ricomincia_con_wild() {
        let mut g = GymProgress::new();
        for _ in 0..WILDS_PER_TRAINER { g.record_wild_defeated(); }
        g.record_trainer_defeated();
        assert_eq!(g.next_opponent(), NextOpponent::Wild);
    }

    #[test]
    fn schema_completo_3w_t_3w_t_3w_capo() {
        let mut g = GymProgress::new();
        // 3 wild → trainer 1
        for _ in 0..3 { assert_eq!(g.next_opponent(), NextOpponent::Wild); g.record_wild_defeated(); }
        assert_eq!(g.next_opponent(), NextOpponent::Trainer);
        g.record_trainer_defeated();
        // 3 wild → trainer 2
        for _ in 0..3 { assert_eq!(g.next_opponent(), NextOpponent::Wild); g.record_wild_defeated(); }
        assert_eq!(g.next_opponent(), NextOpponent::Trainer);
        g.record_trainer_defeated();
        // 3 wild → trainer 3
        for _ in 0..3 { assert_eq!(g.next_opponent(), NextOpponent::Wild); g.record_wild_defeated(); }
        assert_eq!(g.next_opponent(), NextOpponent::Trainer);
        g.record_trainer_defeated();
        // dopo il 3° trainer → capopalestra (nessun wild intermedio)
        assert_eq!(g.next_opponent(), NextOpponent::GymLeader);
    }

    #[test]
    fn dopo_3_trainer_il_prossimo_e_capopalestra() {
        let mut g = GymProgress::new();
        for _ in 0..TRAINERS_PER_GYM {
            for _ in 0..WILDS_PER_TRAINER { g.record_wild_defeated(); }
            g.record_trainer_defeated();
        }
        assert_eq!(g.next_opponent(), NextOpponent::GymLeader);
    }

    #[test]
    fn pokecenter_limit() {
        let mut g = GymProgress::new();
        for _ in 0..POKECENTER_MAX_PER_GYM {
            assert!(g.use_pokecenter().is_ok());
        }
        assert_eq!(g.use_pokecenter(), Err(GymError::PokecenterLimitReached));
    }

    #[test]
    fn advance_resetta_contatori() {
        let mut g = GymProgress::new();
        g.record_wild_defeated();
        g.record_trainer_defeated();
        g.use_pokecenter().unwrap();
        g.advance().unwrap();
        assert_eq!(g.gym_index, 1);
        assert_eq!(g.trainers_defeated, 0);
        assert_eq!(g.wilds_since_last_trainer, 0);
        assert_eq!(g.pokecenter_uses, 0);
    }

    #[test]
    fn badges_avanzano_con_gym_index() {
        let mut g = GymProgress::new();
        g.advance().unwrap();
        g.advance().unwrap();
        assert_eq!(g.badges(), 2);
    }

    #[test]
    fn advance_fallisce_oltre_ottava_palestra() {
        let mut g = GymProgress::new();
        for _ in 0..TOTAL_GYMS - 1 {
            g.advance().unwrap();
        }
        assert_eq!(g.advance(), Err(GymError::AlreadyComplete));
    }
}
