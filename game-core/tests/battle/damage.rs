use game_core::battle::damage::calculate_damage;
use game_core::battle::rng::Rng;
use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn rng() -> Rng { Rng::new(0) }

fn charmander() -> Pokemon {
    Pokemon::new("Charmander", Type::Fire, None, Stats::new(39, 52, 43, 60, 50, 65), 62, 50)
}

fn squirtle() -> Pokemon {
    Pokemon::new("Squirtle", Type::Water, None, Stats::new(44, 48, 65, 50, 64, 43), 63, 50)
}

fn tackle() -> Move { Move::new("Tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35) }
fn surf() -> Move { Move::new("Surf", Type::Water, MoveCategory::Special, 90, 100, 15) }
fn growl() -> Move { Move::new("Growl", Type::Normal, MoveCategory::Status, 0, 100, 40) }

#[test]
fn danno_base_positivo() {
    let res = calculate_damage(&charmander(), &squirtle(), &tackle(), &mut rng());
    assert!(res.damage > 0);
}

#[test]
fn mossa_status_fa_zero_danno() {
    let res = calculate_damage(&charmander(), &squirtle(), &growl(), &mut rng());
    assert_eq!(res.damage, 0);
    assert!(!res.is_crit);
}

#[test]
fn stab_aumenta_il_danno_rispetto_a_neutro() {
    let atk = squirtle();
    let def = charmander();
    let dmg_stab = calculate_damage(&atk, &def, &surf(), &mut Rng::new(1)).damage;
    let dmg_no_stab = calculate_damage(&atk, &def, &tackle(), &mut Rng::new(1)).damage;
    assert!(dmg_stab > dmg_no_stab);
}

#[test]
fn super_efficace_fa_piu_danno_del_neutro() {
    let water_gun = Move::new("Water Gun", Type::Water, MoveCategory::Special, 40, 100, 25);
    let atk = squirtle();
    let def = charmander();
    let dmg_se = calculate_damage(&atk, &def, &water_gun, &mut Rng::new(1)).damage;
    let dmg_neutro = calculate_damage(&atk, &def, &tackle(), &mut Rng::new(1)).damage;
    assert!(dmg_se > dmg_neutro);
    assert_eq!(calculate_damage(&atk, &def, &water_gun, &mut Rng::new(1)).effectiveness, 2.0);
}

#[test]
fn non_molto_efficace_fa_meno_danno_del_neutro() {
    let ember = Move::new("Ember", Type::Fire, MoveCategory::Special, 40, 100, 25);
    let atk = charmander();
    let def = squirtle();
    let dmg_nme = calculate_damage(&atk, &def, &ember, &mut Rng::new(1)).damage;
    let dmg_neutro = calculate_damage(&atk, &def, &tackle(), &mut Rng::new(1)).damage;
    assert!(dmg_nme < dmg_neutro);
    assert_eq!(calculate_damage(&atk, &def, &ember, &mut Rng::new(1)).effectiveness, 0.5);
}

#[test]
fn crit_aumenta_il_danno() {
    let tackle = tackle();
    let atk = charmander();
    let def = squirtle();
    let no_crit = calculate_damage(&atk, &def, &tackle, &mut Rng::new(1));
    let crit = calculate_damage(&atk, &def, &tackle, &mut Rng::new(0));
    if crit.is_crit {
        assert!(crit.damage > no_crit.damage);
    } else {
        let mut found = false;
        for seed in 0u64..1000 {
            let res = calculate_damage(&atk, &def, &tackle, &mut Rng::new(seed));
            if res.is_crit {
                let no = calculate_damage(&atk, &def, &tackle, &mut Rng::new(seed + 1));
                if !no.is_crit {
                    assert!(res.damage > no.damage);
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Nessun seed crit trovato in 1000 tentativi");
    }
}

#[test]
fn danno_deterministico_stesso_seed() {
    let atk = charmander();
    let def = squirtle();
    let d1 = calculate_damage(&atk, &def, &tackle(), &mut Rng::new(42)).damage;
    let d2 = calculate_damage(&atk, &def, &tackle(), &mut Rng::new(42)).damage;
    assert_eq!(d1, d2);
}

#[test]
fn danno_non_supera_hp_massimi_avversario() {
    let dmg = calculate_damage(&charmander(), &squirtle(), &tackle(), &mut rng()).damage;
    assert!(dmg <= squirtle().max_hp());
}

// ── Resistenze forti (ex-immunità) ───────────────────────────────────────────

/// Elettrico vs Terra: effectiveness 0.2, danno minimo 1.
#[test]
fn elettrico_vs_terra_danno_ridotto() {
    let pikachu = Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50);
    let geodude = Pokemon::new("Geodude", Type::Rock, Some(Type::Ground), Stats::new(40, 80, 100, 30, 30, 20), 86, 50);
    let thunderbolt = Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15);
    let res = calculate_damage(&pikachu, &geodude, &thunderbolt, &mut rng());
    assert_eq!(res.effectiveness, 0.2);
    assert!(res.damage >= 1, "danno minimo 1 anche con resistenza forte");
}

/// Normale vs Fantasma: effectiveness 0.2, danno minimo 1.
#[test]
fn normale_vs_fantasma_danno_ridotto() {
    let rattata = Pokemon::new("Rattata", Type::Normal, None, Stats::new(30, 56, 35, 25, 35, 72), 51, 50);
    let gastly = Pokemon::new("Gastly", Type::Ghost, Some(Type::Poison), Stats::new(30, 35, 30, 100, 35, 80), 62, 50);
    let res = calculate_damage(&rattata, &gastly, &tackle(), &mut rng());
    assert_eq!(res.effectiveness, 0.2);
    assert!(res.damage >= 1, "danno minimo 1 anche con resistenza forte");
}

/// Con resistenza forte (ex-immunità) il turno continua normalmente —
/// il danno non è mai 0 quindi TurnResult non è distorto.
#[test]
fn resistenza_forte_non_fa_terminare_il_turno_prematuramente() {
    use game_core::battle::rng::Rng;
    use game_core::battle::turn::{execute_turn, TurnAction, TurnResult};
    let mut pikachu = Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50);
    pikachu.add_move(Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15));
    let mut geodude = Pokemon::new("Geodude", Type::Rock, Some(Type::Ground), Stats::new(40, 80, 100, 30, 30, 20), 86, 50);
    let foe_move = tackle();
    let outcome = execute_turn(&mut pikachu, &mut geodude, TurnAction::UseMove(0), &foe_move, &mut Rng::new(1));

    assert!(outcome.enemy_hit.damage >= 1, "Thunderbolt su Geodude deve fare almeno 1 danno");
    assert!(!geodude.is_fainted(), "Geodude non deve essere KO");
    assert!(outcome.player_hit.damage > 0, "Geodude deve contrattaccare");
    assert_eq!(outcome.result, TurnResult::Ongoing, "il turno deve continuare");
}

/// Doppio tipo con ex-immunità: il prodotto delle effectiveness non può scendere sotto 0.2.
/// Es. Elettrico vs Acqua/Terra → 2.0 * 0.2 = 0.4 (non 0.0 come in origine).
#[test]
fn doppio_tipo_con_resistenza_forte_non_azzera_danno() {
    let pikachu = Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50);
    let wooper = Pokemon::new("Wooper", Type::Water, Some(Type::Ground), Stats::new(55, 45, 45, 25, 25, 15), 73, 50);
    let thunderbolt = Move::new("Thunderbolt", Type::Electric, MoveCategory::Special, 90, 100, 15);
    let res = calculate_damage(&pikachu, &wooper, &thunderbolt, &mut rng());
    // Water×Electric = 2.0, Ground×Electric = 0.2 → prodotto 0.4
    assert_eq!(res.effectiveness, 0.4);
    assert!(res.damage >= 1);
}

/// Mossa Status non beneficia del minimo 1: deve rimanere 0.
#[test]
fn mossa_status_rimane_zero_nonostante_minimo_danno() {
    let res = calculate_damage(&charmander(), &squirtle(), &growl(), &mut rng());
    assert_eq!(res.damage, 0, "le mosse Status non devono mai fare danno");
}

/// ensure_damage_move garantisce che un Pokémon con sole mosse Status possa sempre colpire.
#[test]
fn ensure_damage_move_aggiunge_tackle_se_solo_status() {
    let mut p = Pokemon::new("TestMon", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, 10);
    p.add_move(Move::new("Growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    p.add_move(Move::new("Leer",  Type::Normal, MoveCategory::Status, 0, 100, 30));
    p.ensure_damage_move();
    assert!(p.moves.iter().any(|m| m.deals_damage()), "deve avere almeno una mossa che fa danno");
    let foe = Pokemon::new("Foe", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, 10);
    let tackle = p.moves.iter().find(|m| m.deals_damage()).unwrap().clone();
    let res = calculate_damage(&p, &foe, &tackle, &mut rng());
    assert!(res.damage >= 1, "il tackle di fallback deve fare danno");
}
