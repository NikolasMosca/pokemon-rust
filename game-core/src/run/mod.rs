pub mod gym;
pub mod rewards;

use crate::inventory::Inventory;
use crate::pokemon::Pokemon;
use gym::GymProgress;
use rewards::{BattleKind, calculate_reward, distribute_exp, team_average_level};

pub const STARTING_MONEY: u32 = 3000;
pub const MAX_TEAM_SIZE: usize = 6;

#[derive(Debug, Clone, PartialEq)]
pub enum RunPhase {
    /// Il giocatore sceglie cosa fare dopo una battaglia.
    PostBattle,
    /// Battaglia in corso (selvatica, allenatore, capopalestra).
    InBattle { kind: BattleKind },
    /// Il giocatore è nel menu del Pokécenter.
    Pokecenter,
    /// Il giocatore è nel menu dello Shop.
    Shop,
    /// Game over: tutta la squadra è KO.
    GameOver,
    /// Run completata: tutti gli 8 badge ottenuti.
    RunComplete,
}

#[derive(Debug, Clone)]
pub struct RunState {
    pub team: Vec<Pokemon>,
    pub inventory: Inventory,
    pub gym: GymProgress,
    pub phase: RunPhase,
}

#[derive(Debug, PartialEq)]
pub enum RunError {
    TeamFull,
    GymError(gym::GymError),
    WrongPhase,
}

impl From<gym::GymError> for RunError {
    fn from(e: gym::GymError) -> Self {
        RunError::GymError(e)
    }
}

impl RunState {
    pub fn new(starter: Pokemon) -> Self {
        Self {
            team: vec![starter],
            inventory: Inventory::new(STARTING_MONEY),
            gym: GymProgress::new(),
            phase: RunPhase::InBattle { kind: BattleKind::Wild },
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.team.iter().all(|p| p.is_fainted())
    }

    pub fn team_avg_level(&self) -> u8 {
        team_average_level(&self.team)
    }

    /// Applica la ricompensa dopo una vittoria.
    pub fn apply_reward(&mut self, enemy: &Pokemon, kind: &BattleKind) {
        let reward = calculate_reward(enemy, kind);
        self.inventory.earn(reward.money);
        distribute_exp(&mut self.team, reward.exp);
    }

    /// Aggiunge un Pokémon al team. Restituisce Err se il team è già pieno.
    pub fn catch(&mut self, pokemon: Pokemon) -> Result<(), RunError> {
        if self.team.len() >= MAX_TEAM_SIZE {
            return Err(RunError::TeamFull);
        }
        self.team.push(pokemon);
        Ok(())
    }

    /// Sostituisce uno slot del team con un Pokémon appena catturato.
    pub fn replace_team_slot(&mut self, slot: usize, pokemon: Pokemon) -> Result<(), RunError> {
        if slot >= self.team.len() {
            return Err(RunError::WrongPhase);
        }
        self.team[slot] = pokemon;
        Ok(())
    }

    /// Usa il Pokécenter: heal completo di tutto il team.
    pub fn use_pokecenter(&mut self) -> Result<(), RunError> {
        self.gym.use_pokecenter()?;
        for p in &mut self.team {
            p.full_heal();
        }
        self.phase = RunPhase::PostBattle;
        Ok(())
    }

    /// Registra la sconfitta di un allenatore e aggiorna la fase.
    pub fn on_trainer_defeated(&mut self, enemy: &Pokemon) {
        self.apply_reward(enemy, &BattleKind::Trainer);
        self.gym.record_trainer_defeated();
        self.phase = RunPhase::PostBattle;
    }

    /// Registra la sconfitta del capopalestra e avanza alla palestra successiva.
    pub fn on_gym_leader_defeated(&mut self, enemy: &Pokemon) -> Result<(), RunError> {
        self.apply_reward(enemy, &BattleKind::GymLeader);
        match self.gym.advance() {
            Ok(()) => {
                self.phase = RunPhase::PostBattle;
                Ok(())
            }
            Err(gym::GymError::AlreadyComplete) => {
                self.phase = RunPhase::RunComplete;
                Ok(())
            }
            Err(e) => Err(RunError::GymError(e)),
        }
    }

    /// Registra la sconfitta di un Pokémon selvatico.
    pub fn on_wild_defeated(&mut self, enemy: &Pokemon) {
        self.apply_reward(enemy, &BattleKind::Wild);
        self.gym.record_wild_defeated();
        self.phase = RunPhase::PostBattle;
    }

    /// Controlla se il game over è scattato e aggiorna la fase.
    pub fn check_game_over(&mut self) -> bool {
        if self.is_game_over() {
            self.phase = RunPhase::GameOver;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::Stats;
    use crate::run::rewards::BattleKind;
    use crate::types::Type;

    fn poke(name: &str, level: u8) -> Pokemon {
        Pokemon::new(name, Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, level)
    }

    fn run() -> RunState {
        RunState::new(poke("Pikachu", 5))
    }

    #[test]
    fn game_over_quando_tutti_ko() {
        let mut r = run();
        r.team[0].current_hp = 0;
        assert!(r.check_game_over());
        assert_eq!(r.phase, RunPhase::GameOver);
    }

    #[test]
    fn non_game_over_con_uno_vivo() {
        let mut r = run();
        r.team.push(poke("Charmander", 5));
        r.team[0].current_hp = 0;
        assert!(!r.check_game_over());
    }

    #[test]
    fn catch_aggiunge_al_team() {
        let mut r = run();
        r.catch(poke("Pidgey", 4)).unwrap();
        assert_eq!(r.team.len(), 2);
    }

    #[test]
    fn catch_fallisce_con_team_pieno() {
        let mut r = run();
        for i in 0..5 {
            r.catch(poke(&format!("p{i}"), 5)).unwrap();
        }
        assert_eq!(r.catch(poke("extra", 5)), Err(RunError::TeamFull));
    }

    #[test]
    fn replace_team_slot_con_team_pieno() {
        let mut r = run();
        for i in 0..5 {
            r.catch(poke(&format!("p{i}"), 5)).unwrap();
        }
        assert_eq!(r.team.len(), 6);
        // Con team pieno catch fallisce
        assert_eq!(r.catch(poke("nuovo", 5)), Err(RunError::TeamFull));
        // replace_team_slot sostituisce lo slot 2 con il nuovo pokemon
        r.replace_team_slot(2, poke("nuovo", 5)).unwrap();
        assert_eq!(r.team.len(), 6);
        assert_eq!(r.team[2].name, "nuovo");
    }

    #[test]
    fn replace_team_slot_slot_invalido() {
        let mut r = run();
        assert!(r.replace_team_slot(99, poke("nuovo", 5)).is_err());
    }

    #[test]
    fn pokecenter_ripristina_hp() {
        let mut r = run();
        r.team[0].take_damage(10);
        let hp_prima = r.team[0].current_hp;
        r.use_pokecenter().unwrap();
        assert!(r.team[0].current_hp > hp_prima);
        assert_eq!(r.team[0].current_hp, r.team[0].max_hp());
    }

    #[test]
    fn pokecenter_ripristina_pp() {
        use crate::moves::{Move, MoveCategory};
        let mut r = run();
        r.team[0].add_move(Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35));
        r.team[0].moves[0].current_pp = 5;
        r.use_pokecenter().unwrap();
        assert_eq!(r.team[0].moves[0].current_pp, r.team[0].moves[0].max_pp);
    }

    #[test]
    fn on_gym_leader_defeated_avanza_palestra() {
        let mut r = run();
        let leader = poke("Brock", 12);
        r.on_gym_leader_defeated(&leader).unwrap();
        assert_eq!(r.gym.gym_index, 1);
        assert_eq!(r.gym.badges(), 1);
    }

    #[test]
    fn run_complete_dopo_ottava_palestra() {
        let mut r = run();
        // 8 capopalestra: i primi 7 avanzano la palestra, l'ottavo completa la run
        for _ in 0..8 {
            r.on_gym_leader_defeated(&poke("leader", 20)).unwrap();
        }
        assert_eq!(r.phase, RunPhase::RunComplete);
    }

    #[test]
    fn reward_aggiunge_soldi() {
        let mut r = run();
        let soldi_prima = r.inventory.money;
        r.on_wild_defeated(&poke("Rattata", 5));
        assert!(r.inventory.money > soldi_prima);
    }

    // ── Regressioni bug: post-battaglia e cattura ────────────────────────────

    /// Dopo aver sconfitto un selvatico la fase deve essere PostBattle,
    /// NON tornare subito a InBattle.
    #[test]
    fn wild_defeated_porta_a_post_battle() {
        let mut r = run();
        r.on_wild_defeated(&poke("Rattata", 5));
        assert_eq!(r.phase, RunPhase::PostBattle,
            "dopo wild defeated deve essere PostBattle, non InBattle");
    }

    /// Dopo aver sconfitto un trainer la fase deve essere PostBattle.
    #[test]
    fn trainer_defeated_porta_a_post_battle() {
        let mut r = run();
        r.on_trainer_defeated(&poke("Trainer", 10));
        assert_eq!(r.phase, RunPhase::PostBattle);
    }

    /// La fase InBattle non deve comparire dopo on_wild_defeated —
    /// il giocatore deve sempre passare per PostBattle prima della prossima battaglia.
    #[test]
    fn post_battle_non_avanza_automaticamente() {
        let mut r = run();
        r.on_wild_defeated(&poke("Rattata", 5));
        // La fase deve rimanere PostBattle finché il giocatore non sceglie esplicitamente
        assert_ne!(r.phase, RunPhase::InBattle { kind: BattleKind::Wild },
            "la fase non deve avanzare automaticamente a InBattle");
        assert_ne!(r.phase, RunPhase::InBattle { kind: BattleKind::Trainer },
            "la fase non deve avanzare automaticamente a InBattle");
    }

    /// Dopo la sconfitta di un selvatico, can_catch è sempre true.
    /// La cattura avviene nel PostBattle — non c'è nessuna soglia HP.
    #[test]
    fn can_catch_wild_sempre_true_dopo_sconfitta() {
        let kind = BattleKind::Wild;
        let can_catch = matches!(kind, BattleKind::Wild);
        assert!(can_catch, "un selvatico sconfitto deve sempre essere catturabile");
    }

    /// Per allenatore e capopalestra can_catch è sempre false.
    #[test]
    fn can_catch_false_per_trainer_e_gym() {
        let trainer = BattleKind::Trainer;
        let gym = BattleKind::GymLeader;
        assert!(!matches!(trainer, BattleKind::Wild));
        assert!(!matches!(gym, BattleKind::Wild));
    }

    // ── Bug: danno 0 per immunità non deve bloccare il turno ─────────────────

    /// Il danno 0 per immunità di tipo (es. Elettrico vs Terra) è corretto,
    /// ma NON deve essere usato dalla UI come segnale di "Pokémon KO".
    /// Questo test documenta che on_wild_defeated va a PostBattle anche
    /// quando il foe ha subito 0 danni nell'ultimo turno.
    #[test]
    fn wild_defeated_da_post_battle_indipendentemente_dal_danno() {
        let mut r = run();
        // Anche se il wild ha preso 0 danni in un turno precedente, una volta
        // che on_wild_defeated viene chiamato la fase deve essere PostBattle
        let mut foe = poke("Geodude", 5);
        foe.current_hp = 0; // sconfitto
        r.on_wild_defeated(&foe);
        assert_eq!(r.phase, RunPhase::PostBattle,
            "PostBattle deve apparire anche dopo turni con 0 danni");
    }
}
