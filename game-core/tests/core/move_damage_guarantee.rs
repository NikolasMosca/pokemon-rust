/// Regressione: i Pokémon generati (player e nemico) possono ricevere dal generatore
/// solo mosse Status (es. growl, sand-attack, string-shot) che hanno power=0.
/// In quel caso execute_turn non infligge mai danno → la battaglia non finisce mai.
///
/// Invariante da garantire: dopo la costruzione, ogni Pokémon deve avere
/// almeno una mossa con deals_damage() == true.
/// Se le mosse disponibili sono tutte Status, si aggiunge Tackle come fallback.

use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn status_only_pokemon() -> Pokemon {
    let mut p = Pokemon::new("sandshrew", Type::Ground, None,
        Stats::new(50, 75, 85, 20, 30, 40), 65, 5);
    p.add_move(Move::new("sand-attack", Type::Ground, MoveCategory::Status, 0, 100, 15));
    p.add_move(Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    p
}

fn mixed_pokemon() -> Pokemon {
    let mut p = Pokemon::new("charmander", Type::Fire, None,
        Stats::new(39, 52, 43, 60, 50, 65), 62, 5);
    p.add_move(Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    p.add_move(Move::new("scratch", Type::Normal, MoveCategory::Physical, 40, 100, 35));
    p
}

/// deals_damage() deve essere false per mosse Status con power=0.
#[test]
fn mossa_status_non_deals_damage() {
    let growl = Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40);
    assert!(!growl.deals_damage(),
        "growl (Status, power=0) non deve deals_damage()");

    let sand_attack = Move::new("sand-attack", Type::Ground, MoveCategory::Status, 0, 100, 15);
    assert!(!sand_attack.deals_damage(),
        "sand-attack (Status, power=0) non deve deals_damage()");
}

/// deals_damage() deve essere true per mosse fisiche/speciali con power>0.
#[test]
fn mossa_fisica_deals_damage() {
    let tackle = Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35);
    assert!(tackle.deals_damage(), "tackle (Physical, power=40) deve deals_damage()");

    let ember = Move::new("ember", Type::Fire, MoveCategory::Special, 40, 100, 25);
    assert!(ember.deals_damage(), "ember (Special, power=40) deve deals_damage()");
}

/// BUG REPLICATO: un Pokémon con solo mosse Status non può mai vincere una battaglia.
/// Questo test documenta il problema: has_damage_move() è false.
#[test]
fn pokemon_con_solo_status_non_ha_mossa_danno() {
    let p = status_only_pokemon();
    let has_damage = p.moves.iter().any(|m| m.deals_damage());
    assert!(!has_damage,
        "precondizione: sandshrew con solo Status non ha mosse che fanno danno — questo è il bug");
}

/// Un Pokémon con mosse miste ha almeno una mossa che fa danno.
#[test]
fn pokemon_misto_ha_almeno_una_mossa_danno() {
    let p = mixed_pokemon();
    let has_damage = p.moves.iter().any(|m| m.deals_damage());
    assert!(has_damage,
        "charmander con scratch deve avere almeno una mossa che fa danno");
}

/// INVARIANTE da garantire dopo il fix:
/// Dopo ensure_damage_move(), il Pokémon deve avere almeno una mossa che fa danno.
/// Testa la funzione che il generatore deve chiamare su ogni Pokémon costruito.
#[test]
fn ensure_damage_move_aggiunge_tackle_se_solo_status() {
    let mut p = status_only_pokemon();
    // precondizione: nessuna mossa danno
    assert!(!p.moves.iter().any(|m| m.deals_damage()));

    // applica il fix
    p.ensure_damage_move();

    // postcondizione: almeno una mossa danno
    assert!(p.moves.iter().any(|m| m.deals_damage()),
        "dopo ensure_damage_move() il Pokémon deve avere almeno una mossa che fa danno");
}

/// ensure_damage_move() NON aggiunge mosse se ce n'è già una che fa danno.
#[test]
fn ensure_damage_move_non_aggiunge_se_gia_presente() {
    let mut p = mixed_pokemon();
    let mosse_prima = p.moves.len();
    p.ensure_damage_move();
    assert_eq!(p.moves.len(), mosse_prima,
        "ensure_damage_move() non deve aggiungere mosse se ce n'è già una che fa danno");
}

/// ensure_damage_move() NON aggiunge mosse se il Pokémon ha già 4 mosse e almeno una fa danno.
#[test]
fn ensure_damage_move_non_aggiunge_oltre_quattro() {
    let mut p = Pokemon::new("test", Type::Normal, None,
        Stats::new(45, 49, 49, 65, 65, 45), 64, 5);
    p.add_move(Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35));
    p.add_move(Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    p.add_move(Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    p.add_move(Move::new("growl", Type::Normal, MoveCategory::Status, 0, 100, 40));
    assert_eq!(p.moves.len(), 4);
    p.ensure_damage_move();
    assert_eq!(p.moves.len(), 4,
        "con 4 mosse e una che fa danno, non deve aggiungere nulla");
}
