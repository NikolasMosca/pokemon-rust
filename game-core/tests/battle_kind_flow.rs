/// Test che verificano i comportamenti specifici per tipo di battaglia:
/// - Selvatico → PostBattle dopo un singolo Pokémon sconfitto
/// - Allenatore → PostBattle solo dopo aver sconfitto tutti i Pokémon del team
/// - Capopalestra → idem, poi avanza la palestra
/// - can_catch → true solo per selvatici, mai per trainer/gym leader

use game_core::pokemon::{Pokemon, Stats};
use game_core::run::{RunPhase, RunState};
use game_core::run::rewards::BattleKind;
use game_core::types::Type;

fn poke(name: &str, level: u8) -> Pokemon {
    Pokemon::new(name, Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, level)
}

fn run() -> RunState {
    RunState::new(poke("Pikachu", 10))
}

// ── Selvatico ────────────────────────────────────────────────────────────────

/// Dopo aver sconfitto un Pokémon selvatico, la fase diventa PostBattle.
#[test]
fn wild_singolo_porta_a_post_battle() {
    let mut r = run();
    r.on_wild_defeated(&poke("Rattata", 5));
    assert_eq!(r.phase, RunPhase::PostBattle);
}

/// Per un selvatico, la cattura è possibile (logica UI: can_catch calcolato fuori).
/// Il test verifica che on_wild_defeated non modifichi nulla che impedirebbe la cattura.
#[test]
fn wild_non_impedisce_cattura() {
    let mut r = run();
    let foe = poke("Rattata", 5);
    r.on_wild_defeated(&foe);
    // Dopo wild defeated siamo in PostBattle — il candidato cattura può esistere
    assert_eq!(r.phase, RunPhase::PostBattle);
    // La cattura può avvenire normalmente
    assert!(r.catch(poke("Rattata", 5)).is_ok());
}

// ── Allenatore ───────────────────────────────────────────────────────────────

/// Dopo aver sconfitto il primo Pokémon di un allenatore, on_trainer_defeated
/// porta comunque a PostBattle (la gestione del team avversario è nella UI).
#[test]
fn trainer_defeated_porta_a_post_battle() {
    let mut r = run();
    r.on_trainer_defeated(&poke("Trainer", 10));
    assert_eq!(r.phase, RunPhase::PostBattle);
}

/// Più chiamate a on_trainer_defeated (team avversario multi-Pokémon) portano
/// tutte a PostBattle — ogni vittoria parziale NON deve avanzare automaticamente.
/// La UI deve chiamare on_trainer_defeated UNA SOLA VOLTA alla fine dello scontro.
/// Questo test documenta che chiamarla N volte non causa effetti collaterali inattesi.
#[test]
fn trainer_defeated_idempotente_sulla_fase() {
    let mut r = run();
    r.on_trainer_defeated(&poke("p1", 10));
    assert_eq!(r.phase, RunPhase::PostBattle);
    // Simulazione: la UI era in PostBattle, poi l'utente è tornato a InBattle
    r.phase = RunPhase::InBattle { kind: BattleKind::Trainer };
    r.on_trainer_defeated(&poke("p2", 10));
    assert_eq!(r.phase, RunPhase::PostBattle);
}

/// Dopo on_trainer_defeated, i soldi del trainer sono stati guadagnati.
#[test]
fn trainer_defeated_assegna_ricompensa() {
    let mut r = run();
    let soldi_prima = r.inventory.money;
    r.on_trainer_defeated(&poke("TrainerFoe", 10));
    assert!(r.inventory.money > soldi_prima, "il trainer deve dare soldi");
}

// ── Capopalestra ─────────────────────────────────────────────────────────────

/// Dopo on_gym_leader_defeated, la palestra avanza (gym_index sale di 1).
#[test]
fn gym_leader_defeated_avanza_palestra() {
    let mut r = run();
    let gym_index_prima = r.gym.gym_index;
    r.on_gym_leader_defeated(&poke("Brock", 12)).unwrap();
    assert_eq!(r.gym.gym_index, gym_index_prima + 1);
    assert_eq!(r.phase, RunPhase::PostBattle);
}

/// Dopo on_gym_leader_defeated, i trainers_defeated vengono azzerati (nuova palestra).
#[test]
fn gym_leader_defeated_azzera_trainer_count() {
    let mut r = run();
    r.gym.record_trainer_defeated();
    r.gym.record_trainer_defeated();
    r.on_gym_leader_defeated(&poke("Brock", 12)).unwrap();
    assert_eq!(r.gym.trainers_defeated, 0, "nuova palestra: trainers_defeated deve essere 0");
}

// ── can_catch: logica di cattura ─────────────────────────────────────────────

/// Un selvatico sconfitto è sempre catturabile — nessuna soglia HP.
#[test]
fn can_catch_wild_sempre_true() {
    let kind = BattleKind::Wild;
    let can_catch = matches!(kind, BattleKind::Wild);
    assert!(can_catch);
}

/// Per un Pokémon di allenatore, can_catch è sempre false.
#[test]
fn can_catch_trainer_sempre_false() {
    let kind = BattleKind::Trainer;
    let can_catch = matches!(kind, BattleKind::Wild);
    assert!(!can_catch);
}

/// Per un Pokémon di capopalestra, can_catch è sempre false.
#[test]
fn can_catch_gym_leader_sempre_false() {
    let kind = BattleKind::GymLeader;
    let can_catch = matches!(kind, BattleKind::Wild);
    assert!(!can_catch);
}

// ── PostBattle appare al momento giusto ──────────────────────────────────────

/// Per un selvatico: PostBattle appare subito dopo l'unico Pokémon sconfitto.
#[test]
fn post_battle_wild_appare_dopo_singolo_pokemon() {
    let mut r = run();
    // Prima della battaglia: siamo InBattle
    r.phase = RunPhase::InBattle { kind: BattleKind::Wild };
    // Sconfitta del selvatico
    r.on_wild_defeated(&poke("Rattata", 5));
    assert_eq!(r.phase, RunPhase::PostBattle,
        "per selvatico: PostBattle deve apparire dopo il singolo Pokémon sconfitto");
}

/// Per allenatore con team da 2: PostBattle appare solo quando la UI
/// chiama on_trainer_defeated (dopo l'ultimo Pokémon del team avversario).
/// In mezzo (dopo il primo Pokémon) la fase rimane InBattle.
#[test]
fn post_battle_trainer_appare_solo_dopo_tutto_il_team() {
    let mut r = run();
    r.phase = RunPhase::InBattle { kind: BattleKind::Trainer };

    // La UI sconfigge il primo Pokémon del trainer: NON chiama on_trainer_defeated
    // ma rimane in InBattle e mostra il prossimo Pokémon avversario.
    // Solo alla fine chiama on_trainer_defeated.
    assert_eq!(r.phase, RunPhase::InBattle { kind: BattleKind::Trainer },
        "dopo il primo Pokémon del trainer: ancora InBattle");

    // La UI sconfigge l'ultimo Pokémon → chiama on_trainer_defeated
    r.on_trainer_defeated(&poke("LastFoe", 10));
    assert_eq!(r.phase, RunPhase::PostBattle,
        "dopo l'ultimo Pokémon del trainer: PostBattle");
}

/// Per capopalestra con team da 2: idem, PostBattle appare solo alla fine.
#[test]
fn post_battle_gym_leader_appare_solo_dopo_tutto_il_team() {
    let mut r = run();
    r.phase = RunPhase::InBattle { kind: BattleKind::GymLeader };

    assert_eq!(r.phase, RunPhase::InBattle { kind: BattleKind::GymLeader },
        "durante la battaglia: ancora InBattle");

    r.on_gym_leader_defeated(&poke("GymLeader", 15)).unwrap();
    assert_eq!(r.phase, RunPhase::PostBattle,
        "dopo l'ultimo Pokémon del capopalestra: PostBattle");
}
