use game_core::battle::rng::Rng;
use game_core::battle::turn::{execute_turn, TurnAction, TurnResult};
use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn rng() -> Rng { Rng::new(1) }

fn pikachu() -> Pokemon {
    let mut p = Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50);
    p.add_move(Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15));
    p
}

fn geodude() -> Pokemon {
    Pokemon::new("Geodude", Type::Rock, Some(Type::Ground), Stats::new(40, 80, 100, 30, 30, 20), 86, 50)
}

fn tackle() -> Move { Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35) }

#[test]
fn fuga_termina_il_turno_senza_danno() {
    let mut player = pikachu();
    let mut enemy = geodude();
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::Run, &tackle(), &mut rng());
    assert_eq!(outcome.result, TurnResult::Fled);
    assert_eq!(outcome.enemy_hit.damage, 0);
    assert_eq!(outcome.player_hit.damage, 0);
}

#[test]
fn turno_normale_infligge_danno_a_entrambi() {
    let mut player = pikachu();
    let mut enemy = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert!(outcome.enemy_hit.damage > 0);
    assert!(outcome.player_hit.damage > 0);
    assert_eq!(outcome.result, TurnResult::Ongoing);
}

#[test]
fn attaccante_piu_veloce_vince_prima_di_subire_danno() {
    let mut player = pikachu();
    let mut enemy = Pokemon::new("WeakFoe", Type::Normal, None, Stats::new(1, 10, 5, 10, 5, 10), 50, 1);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert_eq!(outcome.result, TurnResult::PlayerWon);
    assert_eq!(outcome.player_hit.damage, 0);
}

#[test]
fn nemico_piu_veloce_attacca_primo() {
    let mut player = Pokemon::new("SlowPlayer", Type::Normal, None, Stats::new(1, 10, 5, 10, 5, 5), 50, 1);
    player.add_move(tackle());
    let mut enemy = pikachu();
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert_eq!(outcome.result, TurnResult::EnemyWon);
}

#[test]
fn risultato_ongoing_se_entrambi_sopravvivono() {
    let mut player = pikachu();
    let mut enemy = geodude();
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert_eq!(outcome.result, TurnResult::Ongoing);
}

#[test]
fn turno_deterministico_stesso_seed() {
    let mut p1 = pikachu();
    let mut e1 = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let mut p2 = pikachu();
    let mut e2 = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let o1 = execute_turn(&mut p1, &mut e1, TurnAction::UseMove(0), &tackle(), &mut Rng::new(42));
    let o2 = execute_turn(&mut p2, &mut e2, TurnAction::UseMove(0), &tackle(), &mut Rng::new(42));
    assert_eq!(o1.enemy_hit.damage, o2.enemy_hit.damage);
    assert_eq!(o1.player_hit.damage, o2.player_hit.damage);
    assert_eq!(o1.enemy_hit.is_crit, o2.enemy_hit.is_crit);
}

#[test]
fn outcome_espone_flag_crit_e_effectiveness() {
    let mut player = pikachu();
    let mut enemy = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    let _ = outcome.enemy_hit.is_crit;
    let _ = outcome.enemy_hit.effectiveness;
    let _ = outcome.player_hit.is_crit;
}

// ── Regressioni bug: turno e post-battaglia ──────────────────────────────────

/// Se il giocatore più veloce uccide il nemico, il nemico NON deve contrattaccare.
/// Regressione: battle_screen applicava player_hit.damage anche con nemico già KO.
#[test]
fn nemico_ko_non_contrattacca_se_player_piu_veloce() {
    // Player velocissimo, nemico con 1 HP
    let mut player = pikachu(); // speed 90
    let mut enemy = Pokemon::new("WeakFoe", Type::Normal, None, Stats::new(1, 10, 5, 10, 5, 10), 50, 1);
    // enemy ha 1 HP → muore al primo colpo
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());

    assert_eq!(outcome.result, TurnResult::PlayerWon);
    // Il nemico è fainted → NON deve aver inflitto danno al player
    assert_eq!(outcome.player_hit.damage, 0, "il nemico KO non deve contrattaccare");
    assert!(enemy.is_fainted());
}

/// Se il nemico più veloce uccide il giocatore, il giocatore NON deve contrattaccare.
#[test]
fn player_ko_non_contrattacca_se_nemico_piu_veloce() {
    // Nemico velocissimo con attacco enorme, player con 1 HP
    let mut player = Pokemon::new("WeakPlayer", Type::Normal, None, Stats::new(1, 5, 5, 5, 5, 5), 50, 1);
    player.add_move(tackle());
    let strong_tackle = Move::new("HeavySlam", Type::Normal, MoveCategory::Physical, 200, 100, 10);
    let mut enemy = Pokemon::new("FastStrong", Type::Normal, None, Stats::new(100, 150, 50, 50, 50, 200), 200, 50);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &strong_tackle, &mut rng());

    assert_eq!(outcome.result, TurnResult::EnemyWon);
    // Il player è fainted → NON deve aver inflitto danno al nemico
    assert_eq!(outcome.enemy_hit.damage, 0, "il player KO non deve contrattaccare");
    assert!(player.is_fainted());
}

/// La logica UI deve usare player_hit.damage == 0 come segnale che il nemico
/// era già KO quando è stato il suo turno — non deve mostrare il contrattacco.
#[test]
fn outcome_damage_zero_indica_nessun_contrattacco() {
    let mut player = pikachu();
    let mut enemy = Pokemon::new("WeakFoe", Type::Normal, None, Stats::new(1, 10, 5, 10, 5, 10), 50, 1);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());

    // Usato dalla UI per decidere se mostrare la sequenza di contrattacco
    let nemico_ha_contrattaccato = outcome.player_hit.damage > 0;
    assert!(!nemico_ha_contrattaccato, "la UI non deve mostrare il contrattacco se damage == 0");
}

// ── Bug: foe senza mosse ─────────────────────────────────────────────────────

/// Se il foe non ha mosse (fetch API fallito), execute_turn deve comunque
/// completare il turno invece di bloccarsi silenziosamente.
/// Il foe deve usare una mossa di fallback (Struggle/Tackle).
/// Regressione: execute_turn riceveva `enemy_move: &Move` separato dal foe,
/// ma la UI usava `foe.moves.first()?` per ottenere enemy_move — se vuoto,
/// la closure restituiva None e il turno non veniva mai eseguito.
#[test]
fn execute_turn_con_foe_senza_mosse_produce_danno() {
    // Foe senza nessuna mossa aggiunta
    let mut player = pikachu();
    let mut enemy_no_moves = Pokemon::new(
        "FoeNoMoves", Type::Normal, None,
        Stats::new(40, 60, 40, 40, 40, 30), 50, 10
    );
    assert!(enemy_no_moves.moves.is_empty(), "il foe non deve avere mosse");

    // La UI deve fornire una mossa di fallback invece di abortire
    let fallback = tackle();
    let outcome = execute_turn(&mut player, &mut enemy_no_moves, TurnAction::UseMove(0), &fallback, &mut rng());

    // Il turno deve produrre un risultato valido (non bloccarsi)
    assert!(
        outcome.result == TurnResult::PlayerWon
        || outcome.result == TurnResult::EnemyWon
        || outcome.result == TurnResult::Ongoing,
        "il turno deve completarsi anche con foe senza mosse"
    );
    // Il player deve aver subito danno dal fallback (foe non è immune al Normal)
    assert!(outcome.player_hit.damage > 0 || outcome.result == TurnResult::PlayerWon,
        "il foe deve comunque contrattaccare con la mossa di fallback");
}

/// La UI non deve silenziosamente abortire il turno se il foe non ha mosse.
/// Il comportamento atteso: usare una mossa di fallback (Tackle base).
#[test]
fn execute_turn_risultato_sempre_valido() {
    let mut player = pikachu();
    let mut enemy = Pokemon::new("Foe", Type::Normal, None, Stats::new(40, 60, 40, 40, 40, 30), 50, 10);
    // Nessuna mossa aggiunta — simula il caso di fetch API fallito
    let fallback = Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &fallback, &mut rng());
    // Il risultato non può essere Fled (non abbiamo fuggito)
    assert_ne!(outcome.result, TurnResult::Fled);
    // enemy_hit.damage deve essere > 0 (player ha usato Thunderbolt su Normal)
    assert!(outcome.enemy_hit.damage > 0, "il player deve infliggere danno");
}

// ── Bug PP ───────────────────────────────────────────────────────────────────

/// Dopo execute_turn la PP della mossa usata deve essere decrementata di 1.
/// Regressione: execute_turn clonava la mossa senza riscrivere il PP sul player.
#[test]
fn execute_turn_decrementa_pp_player() {
    let mut player = pikachu(); // Thunderbolt ha 15 PP
    let mut enemy = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let pp_prima = player.moves[0].current_pp;
    execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert_eq!(player.moves[0].current_pp, pp_prima - 1, "execute_turn deve decrementare la PP della mossa usata");
}

/// Se il player usa una mossa con 0 PP, execute_turn non deve decrementare sotto zero.
#[test]
fn execute_turn_pp_non_scende_sotto_zero() {
    let mut player = pikachu();
    player.moves[0].current_pp = 0;
    let mut enemy = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());
    assert_eq!(player.moves[0].current_pp, 0, "PP non deve scendere sotto zero");
}

// ── Bug danno 0 non deve interrompere il turno ───────────────────────────────

/// Una mossa di tipo Status ha power=0 → danno=0, ma il turno deve continuare
/// normalmente: il nemico deve ancora contrattaccare se sopravvive.
/// Regressione: la UI usava damage==0 come proxy di "nemico KO" — sbagliato.
#[test]
fn status_move_danno_zero_ma_result_ongoing() {
    // Player più veloce, mossa Status (power=0), nemico sopravvive
    let mut player = Pokemon::new("Player", Type::Normal, None, Stats::new(45, 49, 49, 65, 65, 100), 50, 50);
    player.add_move(Move::new("Growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    let mut enemy = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let outcome = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &tackle(), &mut rng());

    // La mossa Status non infligge danno al nemico
    assert_eq!(outcome.enemy_hit.damage, 0, "una mossa Status deve fare 0 danni");
    // Ma il nemico è ancora vivo e contrattacca
    assert!(!enemy.is_fainted(), "il nemico non deve essere KO dopo una mossa Status");
    // Il risultato NON deve essere PlayerWon — il turno deve continuare
    assert_eq!(outcome.result, TurnResult::Ongoing, "una mossa Status non uccide: result deve essere Ongoing");
    // CHIAVE: player_hit.damage > 0 perché il nemico ha effettivamente contrattaccato
    assert!(outcome.player_hit.damage > 0, "il nemico deve contrattaccare dopo una mossa Status del player");
}
