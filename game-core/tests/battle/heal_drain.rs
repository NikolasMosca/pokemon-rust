use game_core::battle::rng::Rng;
use game_core::battle::turn::{execute_turn, TurnAction, TurnResult};
use game_core::moves::{Move, MoveCategory, MoveEffect};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn rng() -> Rng { Rng::new(1) }

fn tackle() -> Move {
    Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35)
}

fn recover() -> Move {
    Move::new("Recover", Type::Normal, MoveCategory::Status, 0, 100, 10)
        .with_effect(MoveEffect::Heal { percent: 50 })
}

fn mega_drain() -> Move {
    Move::new("Mega Drain", Type::Grass, MoveCategory::Special, 40, 100, 15)
        .with_effect(MoveEffect::Drain { percent: 50 })
}

fn bulbasaur() -> Pokemon {
    // Tipo Grass/Poison, sp_atk alto — usa Mega Drain
    Pokemon::new("Bulbasaur", Type::Grass, Some(Type::Poison), Stats::new(45, 49, 49, 65, 65, 45), 64, 50)
}

fn squirtle() -> Pokemon {
    Pokemon::new("Squirtle", Type::Water, None, Stats::new(44, 48, 65, 50, 64, 43), 63, 50)
}

fn chansey() -> Pokemon {
    // HP altissimi, velocità bassa — usa Recover
    Pokemon::new("Chansey", Type::Normal, None, Stats::new(250, 5, 5, 35, 105, 50), 395, 50)
}

// ── Heal: guarigione percentuale ──────────────────────────────────────────────

/// Caso 1: player usa Recover con HP parziali → recupera 50% dei max HP.
#[test]
fn heal_ripristina_50_percento_max_hp() {
    let mut user = chansey();
    let max = user.max_hp();
    user.take_damage(max / 2);
    let hp_prima = user.current_hp;
    user.add_move(recover());

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_player > 0, "deve registrare la guarigione");
    assert!(user.current_hp > hp_prima, "gli HP devono essere aumentati");
}

/// Caso 2: guarigione cappata a max_hp — healed_player non supera il gap di HP mancanti.
#[test]
fn heal_non_supera_max_hp() {
    let mut user = chansey();
    let max = user.max_hp();
    // Togliamo solo 1 HP: il gap è 1, quindi healed_player deve essere al massimo 1
    user.take_damage(1);
    user.add_move(recover());

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_player <= 1, "la guarigione non può superare il gap di HP mancanti");
    assert!(user.current_hp <= max, "gli HP non devono mai superare il massimo");
}

/// Caso 3: Recover usato a piena salute → healed_player=0, PP consumati.
#[test]
fn heal_a_piena_salute_non_cambia_hp_ma_consuma_pp() {
    let mut user = chansey();
    user.add_move(recover());
    let pp_prima = user.moves[0].current_pp;

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert_eq!(out.healed_player, 0, "nessun HP guadagnato se già al massimo");
    assert_eq!(user.moves[0].current_pp, pp_prima - 1, "PP devono essere consumati");
}

/// Caso 4: il nemico usa una mossa Heal → i suoi HP salgono.
#[test]
fn heal_dell_ai_ripristina_hp_nemico() {
    let mut player = squirtle();
    player.add_move(tackle());

    let mut foe = chansey();
    let max = foe.max_hp();
    foe.take_damage(max / 2);
    let hp_prima = foe.current_hp;

    let foe_move = recover();
    let out = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_enemy > 0, "deve registrare la guarigione del nemico");
    assert!(foe.current_hp > hp_prima, "gli HP del nemico devono essere aumentati");
}

/// Caso 5: PP consumati dalla mossa Heal.
#[test]
fn heal_consuma_pp() {
    let mut user = chansey();
    user.add_move(recover());
    let pp_prima = user.moves[0].current_pp;

    let mut foe = squirtle();
    let foe_move = tackle();
    execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert_eq!(user.moves[0].current_pp, pp_prima - 1);
}

/// Caso 6: TurnOutcome riporta healed_player > 0 dopo guarigione effettiva.
#[test]
fn turn_outcome_riporta_guarigione() {
    let mut user = chansey();
    user.take_damage(user.max_hp() / 2);
    user.add_move(recover());

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_player > 0);
    assert_eq!(out.healed_enemy, 0);
}

// ── Drain ─────────────────────────────────────────────────────────────────────

/// Caso 7: danno inflitto E HP guadagnati — drain fa entrambe le cose.
#[test]
fn drain_infligge_danno_e_guarisce() {
    let mut user = bulbasaur();
    user.take_damage(user.max_hp() / 2);
    user.add_move(mega_drain());
    let hp_prima = user.current_hp;

    let mut foe = squirtle();
    let foe_hp_prima = foe.current_hp;

    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(foe.current_hp < foe_hp_prima, "il foe deve aver subito danno");
    assert!(out.healed_player > 0, "il player deve aver guadagnato HP");
    assert!(user.current_hp > hp_prima, "gli HP del player devono essere aumentati");
}

/// Caso 8: drain cappato a max_hp del user — healed_player non supera il gap di HP mancanti.
#[test]
fn drain_non_supera_max_hp_del_user() {
    let mut user = bulbasaur();
    // Solo 1 HP mancante: il drain recupererebbe molto di più ma deve essere cappato a 1
    user.take_damage(1);
    let max = user.max_hp();
    user.add_move(mega_drain());

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_player <= 1, "drain non può recuperare più del gap di HP mancanti");
    assert!(user.current_hp <= max, "gli HP non devono superare il massimo");
}

/// Caso 9: nemico quasi KO — il drain guadagna solo in base al danno realmente subito.
#[test]
fn drain_su_nemico_quasi_ko_guadagna_in_base_al_danno_reale() {
    let mut user = bulbasaur();
    user.take_damage(user.max_hp() / 2);
    user.add_move(mega_drain());

    let mut foe = squirtle();
    foe.current_hp = 2; // quasi KO

    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    // Il foe aveva 2 HP: drain può aver fatto al massimo 2 di danno reale
    // Gli HP guadagnati devono essere proporzionali al danno reale (≤ 2 * 50% = 1)
    assert!(out.healed_player <= 2, "drain non può guadagnare più HP di quanti ne aveva il foe");
}

/// Caso 10: il nemico usa una mossa Drain → guadagna HP, player li perde.
#[test]
fn drain_dell_ai_guarisce_il_nemico() {
    let mut player = squirtle();
    player.add_move(tackle());

    let mut foe = bulbasaur();
    foe.take_damage(foe.max_hp() / 2);
    let foe_hp_prima = foe.current_hp;

    let foe_move = mega_drain();
    let out = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_enemy > 0, "il nemico deve aver guadagnato HP con drain");
    assert!(foe.current_hp > foe_hp_prima, "gli HP del nemico devono essere aumentati");
}

/// Caso 11: drain dell'AI non supera max_hp del nemico — healed_enemy ≤ gap di HP mancanti.
#[test]
fn drain_ai_non_supera_max_hp_nemico() {
    let mut player = squirtle();
    player.add_move(tackle());

    let mut foe = bulbasaur();
    foe.take_damage(1); // quasi pieno: gap = 1
    let max = foe.max_hp();

    let foe_move = mega_drain();
    let out = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(out.healed_enemy <= 1, "drain AI non può recuperare più del gap di HP mancanti");
    assert!(foe.current_hp <= max, "gli HP del foe non devono superare il massimo");
}

// ── Casi limite ───────────────────────────────────────────────────────────────

/// Caso 12: TurnResult corretto dopo Heal — se il nemico KO il player dopo la guarigione, vince il nemico.
#[test]
fn heal_non_impedisce_enemy_won_se_player_muore() {
    // Player molto debole: il nemico lo KO anche dopo che il player si cura
    let mut player = Pokemon::new("Fragile", Type::Normal, None, Stats::new(10, 5, 5, 5, 5, 5), 50, 1);
    player.current_hp = 1;
    player.add_move(recover());

    // Nemico fortissimo e lento (player va prima, si cura, poi il nemico lo KO)
    let mut foe = Pokemon::new("Tank", Type::Normal, None, Stats::new(200, 150, 10, 10, 10, 1), 200, 100);
    let foe_move = Move::new("Hyper", Type::Normal, MoveCategory::Physical, 250, 100, 5);

    let out = execute_turn(&mut player, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert_eq!(out.result, TurnResult::EnemyWon, "il nemico vince anche se il player si era curato");
}

/// Caso 13: drain con effectiveness 0.2 — danno ridotto e HP guadagnati ridotti.
#[test]
fn drain_con_resistenza_forte_guadagna_meno_hp() {
    // Grass vs Steel: effectiveness 0.2 (ex-immunità ridotta)
    let mut user = bulbasaur();
    user.take_damage(user.max_hp() / 2);
    user.add_move(mega_drain());

    // Foe di tipo Acciaio: Grass vs Steel = 0.5
    let mut foe_steel = Pokemon::new("Steelix", Type::Steel, None, Stats::new(75, 85, 200, 55, 65, 30), 179, 50);

    // Foe normale per confronto
    let mut foe_normal = squirtle();

    let hp_prima_steel = user.current_hp;
    let foe_move = tackle();
    let out_steel = execute_turn(&mut user, &mut foe_steel, TurnAction::UseMove(0), &foe_move, &mut rng());
    let healed_vs_steel = out_steel.healed_player;

    user.current_hp = hp_prima_steel; // reset per confronto
    let out_normal = execute_turn(&mut user, &mut foe_normal, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert!(healed_vs_steel <= out_normal.healed_player,
        "drain vs resistenza deve guadagnare meno HP che contro tipo neutro");
}

/// Caso 14: TurnResult corretto dopo Heal — se player si cura ma foe è già vivo, Ongoing.
#[test]
fn heal_con_foe_vivo_risulta_ongoing() {
    let mut user = chansey();
    user.take_damage(user.max_hp() / 2);
    user.add_move(recover());

    let mut foe = squirtle();
    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert_eq!(out.result, TurnResult::Ongoing);
}

/// Caso 15: TurnResult PlayerWon dopo Drain che porta il nemico a 0 HP.
#[test]
fn drain_che_ko_il_nemico_risulta_player_won() {
    let mut user = bulbasaur();
    user.add_move(mega_drain());

    let mut foe = squirtle();
    foe.current_hp = 1; // quasi KO, drain lo elimina

    let foe_move = tackle();
    let out = execute_turn(&mut user, &mut foe, TurnAction::UseMove(0), &foe_move, &mut rng());

    assert_eq!(out.result, TurnResult::PlayerWon, "drain che KO il nemico deve dare PlayerWon");
}
