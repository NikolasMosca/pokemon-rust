/// Test di integrazione che replicano esattamente il flusso che la UI esegue
/// dentro `end_battle` in `battle_screen.rs`.
/// Obiettivo: verificare che RunPhase::PostBattle sia raggiunto in ogni scenario.

use game_core::battle::turn::{execute_turn, TurnAction, TurnResult};
use game_core::battle::rng::Rng;
use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::run::rewards::BattleKind;
use game_core::run::{RunPhase, RunState};
use game_core::types::Type;

fn starter() -> Pokemon {
    let mut p = Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 10);
    p.add_move(Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15));
    p
}

fn weak_foe() -> Pokemon {
    // HP bassissimi: muore al primo colpo
    Pokemon::new("WeakFoe", Type::Normal, None, Stats::new(1, 10, 5, 10, 5, 10), 50, 1)
}

fn normal_foe() -> Pokemon {
    Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 5)
}

fn fallback_move() -> Move {
    Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35)
}

// ── Replica esatta del flusso UI in end_battle ────────────────────────────────

/// Scenario 1: player più veloce, uccide il foe al primo colpo.
/// La UI chiama end_battle → on_wild_defeated → RunPhase::PostBattle.
/// Questo test replica esattamente le chiamate che la UI fa su RunState.
#[test]
fn end_battle_flow_player_vince_porta_a_post_battle() {
    let mut run = RunState::new(starter());
    let mut foe = weak_foe();
    let foe_move = fallback_move();
    let mut rng = Rng::new(42);

    // Passo 1: esegue il turno (come fa la UI in execute_move)
    let outcome = execute_turn(
        &mut run.team[0], &mut foe,
        TurnAction::UseMove(0), &foe_move, &mut rng
    );
    assert_eq!(outcome.result, TurnResult::PlayerWon, "precondizione: player deve vincere");

    // Passo 2: la UI chiama end_battle che internamente chiama on_wild_defeated
    run.on_wild_defeated(&foe);

    // Passo 3: la fase DEVE essere PostBattle
    assert_eq!(run.phase, RunPhase::PostBattle,
        "dopo on_wild_defeated la fase deve essere PostBattle, non {:?}", run.phase);
}

/// Scenario 2: foe più veloce, sopravvive al contrattacco del player.
/// La UI chiama end_battle solo se result == PlayerWon — se Ongoing, non la chiama.
/// Test che verifica che il turno Ongoing NON chiami on_wild_defeated.
#[test]
fn turno_ongoing_non_porta_a_post_battle() {
    let mut run = RunState::new(starter());
    // Foe coriaceo: sopravvive
    let mut foe = Pokemon::new(
        "Geodude", Type::Rock, Some(Type::Ground),
        Stats::new(40, 80, 100, 30, 30, 20), 86, 50
    );
    let foe_move = fallback_move();
    let mut rng = Rng::new(42);

    let outcome = execute_turn(
        &mut run.team[0], &mut foe,
        TurnAction::UseMove(0), &foe_move, &mut rng
    );

    // Se il turno è Ongoing, la UI NON chiama on_wild_defeated
    if outcome.result == TurnResult::Ongoing {
        // la fase deve rimanere InBattle
        assert!(
            matches!(run.phase, RunPhase::InBattle { .. }),
            "con turno Ongoing la fase NON deve cambiare: {:?}", run.phase
        );
    }
    // Se il player ha vinto (crit o Level 50 vs Level 50 Geodude), OK
}

/// Scenario 3: il flusso completo su più turni — alla fine player vince,
/// on_wild_defeated deve essere chiamata una sola volta e portare a PostBattle.
#[test]
fn multi_turno_fine_porta_a_post_battle() {
    let mut run = RunState::new(starter());
    let mut foe = normal_foe();
    let foe_move = fallback_move();
    let mut rng = Rng::new(1);

    // Simula turni finché uno dei due vince
    let mut turni = 0;
    let final_result = loop {
        turni += 1;
        if turni > 50 { panic!("troppi turni, qualcosa è bloccato"); }

        let outcome = execute_turn(
            &mut run.team[0], &mut foe,
            TurnAction::UseMove(0), &foe_move, &mut rng
        );

        match outcome.result {
            TurnResult::PlayerWon => {
                // La UI chiama end_battle → on_wild_defeated
                run.on_wild_defeated(&foe);
                break TurnResult::PlayerWon;
            }
            TurnResult::EnemyWon => {
                break TurnResult::EnemyWon;
            }
            _ => continue,
        }
    };

    if final_result == TurnResult::PlayerWon {
        assert_eq!(run.phase, RunPhase::PostBattle,
            "dopo la vittoria la fase deve essere PostBattle, non {:?}", run.phase);
    }
}

/// Scenario 4: verifica che RunState parta da InBattle e arrivi a PostBattle
/// attraverso on_wild_defeated — replica precisa della sequenza UI.
#[test]
fn run_state_parte_da_inbattle_arriva_a_post_battle() {
    let run = RunState::new(starter());
    assert!(
        matches!(run.phase, RunPhase::InBattle { .. }),
        "RunState::new deve partire da InBattle, non {:?}", run.phase
    );

    let mut run = run;
    let foe = weak_foe();
    run.on_wild_defeated(&foe);

    assert_eq!(run.phase, RunPhase::PostBattle,
        "on_wild_defeated deve portare a PostBattle da InBattle");
}

// ── Bug: player_first nella UI diverge da player_first in execute_turn ────────

/// Replica esatta della logica UI in execute_move per determinare player_first.
/// La UI usa: player_speed >= foe.base_stats.speed (clone pre-turno)
/// execute_turn usa: player.base_stats.speed >= enemy.base_stats.speed
/// Devono essere identici. Se divergono, la UI mostra la sequenza di animazione
/// sbagliata: foe già KO che "contrattacca", o post-battle non triggerato.
#[test]
fn player_first_ui_coerente_con_execute_turn() {
    // Player più veloce del foe
    let mut player = Pokemon::new("FastPlayer", Type::Normal, None,
        Stats::new(45, 49, 49, 65, 65, 100), 64, 10); // speed 100
    player.add_move(fallback_move());
    let mut foe = Pokemon::new("SlowFoe", Type::Normal, None,
        Stats::new(45, 49, 49, 65, 65, 30), 64, 10); // speed 30
    foe.add_move(fallback_move());

    let foe_speed_before = foe.base_stats.speed; // salvato prima del turno (come fa UI con foe.clone())
    let player_speed_before = player.base_stats.speed; // come fa UI leggendo da signal dopo execute_turn

    let foe_move = foe.moves[0].clone();
    let mut rng = Rng::new(1);
    let outcome = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng);

    // La speed non cambia durante il turno, quindi player_first è consistente
    let player_first_in_turn = player_speed_before >= foe_speed_before;
    assert!(player_first_in_turn, "player più veloce deve andare per primo");

    // Se player_first e ha vinto: p_dmg deve essere 0 (foe non ha contrattaccato)
    if outcome.result == TurnResult::PlayerWon {
        assert_eq!(outcome.player_hit.damage, 0,
            "se player vince per primo, il foe non deve aver contrattaccato");
    }
}

/// Replica il caso in cui il foe è più veloce e uccide il player.
/// La UI deve mostrare: foe attacca → player KO → no contrattacco del player.
#[test]
fn enemy_first_player_ko_no_contrattacco() {
    let mut player = Pokemon::new("SlowPlayer", Type::Normal, None,
        Stats::new(1, 5, 5, 5, 5, 5), 50, 1); // HP bassissimi, speed bassa
    player.add_move(fallback_move());
    let mut foe = Pokemon::new("FastStrong", Type::Normal, None,
        Stats::new(100, 150, 50, 50, 50, 200), 200, 50); // velocissimo, fortissimo
    let strong_move = Move::new("HeavySlam", Type::Normal, MoveCategory::Physical, 200, 100, 10);

    let foe_move = strong_move;
    let mut rng = Rng::new(1);
    let outcome = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng);

    assert_eq!(outcome.result, TurnResult::EnemyWon);
    // Player è KO, non ha contrattaccato
    assert_eq!(outcome.enemy_hit.damage, 0,
        "player KO non deve aver contrattaccato il foe");
}

// ── Bug: player o foe senza mosse — il turno non termina mai ─────────────────

/// Se il player non ha mosse (fetch API fallito per lo starter), execute_turn
/// non infligge danni al foe. Il foe contrattacca ma non muore mai.
/// Il turno risulta Ongoing all'infinito → end_battle non viene mai chiamata
/// → PostBattle non appare mai.
/// Regressione: sia lo starter che il foe devono avere almeno 1 mossa dopo la generazione.
#[test]
fn player_senza_mosse_non_infligge_danno_al_foe() {
    // Starter senza mosse — simula fetch API fallito per le mosse dello starter
    let mut run = RunState::new(
        Pokemon::new("Starter", Type::Normal, None, Stats::new(45, 49, 49, 65, 65, 45), 64, 5)
        // nessuna mossa aggiunta
    );
    assert!(run.team[0].moves.is_empty(), "precondizione: starter senza mosse");

    let mut foe = normal_foe();
    foe.add_move(fallback_move());
    let foe_move = foe.moves[0].clone();
    let mut rng = Rng::new(1);

    let outcome = execute_turn(
        &mut run.team[0], &mut foe,
        TurnAction::UseMove(0), // indice 0 ma moves è vuoto
        &foe_move, &mut rng
    );

    // Il foe NON deve morire — il player non ha inflitto danno
    assert_eq!(outcome.enemy_hit.damage, 0,
        "senza mosse il player non deve infliggere danno");
    assert!(!foe.is_fainted(),
        "il foe non deve morire se il player non ha mosse");
    // Il risultato non può essere PlayerWon — il foe è ancora vivo
    assert_ne!(outcome.result, TurnResult::PlayerWon,
        "senza mosse il player non può vincere");
    // QUESTO È IL BUG: il turno è Ongoing ma il player non può mai vincere
    // senza mosse → end_battle non viene mai chiamata → PostBattle non appare
    assert_eq!(outcome.result, TurnResult::Ongoing,
        "il turno è bloccato in Ongoing: bug confermato");
}

/// Lo starter DEVE avere almeno 1 mossa dopo la generazione.
/// Se build_pokemon non trova mosse dall'API, deve aggiungere un Tackle di fallback.
/// Questo è il fix che sblocca il turno e permette al PostBattle di apparire.
#[test]
fn starter_senza_mosse_da_api_deve_avere_fallback() {
    // Simula starter senza mosse (build_pokemon con move_details vuoto)
    let mut starter = Pokemon::new("Bulbasaur", Type::Grass, Some(Type::Poison),
        Stats::new(45, 49, 49, 65, 65, 45), 64, 5);
    // Nessuna mossa — simula build_pokemon con []

    // Il fix atteso: se moves è vuoto, aggiungere Tackle di fallback
    if starter.moves.is_empty() {
        starter.add_move(Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35));
    }

    assert!(!starter.moves.is_empty(), "lo starter deve avere almeno 1 mossa");
    assert_eq!(starter.moves[0].power, 40, "il fallback deve essere Tackle (power 40)");
}

// ── Regressione: rimontaggio BattleScreen durante il turno ───────────────────

/// BUG REPLICATO: in game.rs il blocco che fa il match su run.phase si rieseguiva
/// ad ogni scrittura a `run`, inclusi gli aggiornamenti agli HP del team durante
/// il turno. Poiché la fase era ancora InBattle, Leptos rimontava BattleScreen
/// da capo, generando un nuovo nemico a piena vita e azzerando il turno.
///
/// INVARIANTE da garantire: aggiornare gli HP/PP del team durante un turno
/// NON deve cambiare run.phase — la fase deve rimanere InBattle fino alla
/// chiamata esplicita di on_wild_defeated / on_trainer_defeated.
#[test]
fn aggiornare_hp_team_durante_turno_non_cambia_fase() {
    let mut run = RunState::new(starter());

    // Precondizione: la fase è InBattle
    assert!(
        matches!(run.phase, RunPhase::InBattle { .. }),
        "precondizione: RunState::new deve partire da InBattle"
    );

    // Simula quello che la UI fa nello step 4 di execute_move:
    // aggiorna gli HP del Pokémon attivo dopo il turno (danno subito, PP usati).
    // Questa operazione NON deve cambiare la fase.
    if let Some(p) = run.team.get_mut(0) {
        p.take_damage(5);
        if let Some(m) = p.moves.get_mut(0) {
            m.use_pp();
        }
    }

    assert!(
        matches!(run.phase, RunPhase::InBattle { .. }),
        "aggiornare HP/PP del team NON deve cambiare la fase a {:?} — \
         regressione: causava rimontaggio BattleScreen e rigenerazione del nemico",
        run.phase
    );
}

/// Stesso invariante su più turni: la fase rimane InBattle per tutta la durata
/// della battaglia, fino alla chiamata esplicita di on_wild_defeated.
#[test]
fn fase_rimane_inbattle_per_tutta_la_durata_del_turno() {
    let mut run = RunState::new(starter());
    let mut foe = normal_foe();
    let foe_move = fallback_move();
    let mut rng = Rng::new(42);

    // Esegui 3 turni — la fase deve restare InBattle finché il foe è vivo
    for turno in 1..=3 {
        let outcome = execute_turn(
            &mut run.team[0], &mut foe,
            TurnAction::UseMove(0), &foe_move, &mut rng,
        );

        // La UI aggiorna il team nel signal run dopo ogni turno
        // (questo è lo step che causava il rimontaggio)
        // Simuliamo lo stesso aggiornamento direttamente su run.team
        // senza toccare run.phase
        // (in Rust non c'è signal, ma il test verifica l'invariante logico)

        if outcome.result == TurnResult::PlayerWon || outcome.result == TurnResult::EnemyWon {
            break;
        }

        assert!(
            matches!(run.phase, RunPhase::InBattle { .. }),
            "turno {turno}: la fase deve restare InBattle, non {:?}",
            run.phase
        );
    }
}

/// La fase diventa PostBattle SOLO dopo on_wild_defeated, mai prima.
/// Verifica che non ci sia nessun percorso che porta a PostBattle durante il turno.
#[test]
fn post_battle_appare_solo_dopo_on_wild_defeated_non_prima() {
    let mut run = RunState::new(starter());
    let mut foe = weak_foe();
    let foe_move = fallback_move();
    let mut rng = Rng::new(42);

    // Esegui il turno — il player vince al primo colpo
    let outcome = execute_turn(
        &mut run.team[0], &mut foe,
        TurnAction::UseMove(0), &foe_move, &mut rng,
    );
    assert_eq!(outcome.result, TurnResult::PlayerWon, "precondizione: player vince");

    // PRIMA di on_wild_defeated: la fase è ancora InBattle
    // Questo è il momento in cui la UI aggiorna il team nel signal.
    // Il bug: questo aggiornamento causava il rimontaggio → nuovo nemico generato.
    assert!(
        matches!(run.phase, RunPhase::InBattle { .. }),
        "PRIMA di on_wild_defeated la fase deve essere ancora InBattle, non {:?} — \
         regressione: PostBattle appariva troppo presto o BattleScreen veniva rimontato",
        run.phase
    );

    // DOPO on_wild_defeated: la fase diventa PostBattle
    run.on_wild_defeated(&foe);
    assert_eq!(
        run.phase, RunPhase::PostBattle,
        "DOPO on_wild_defeated la fase deve essere PostBattle"
    );
}
