/// Test che verificano che la selezione della mossa nemica sia casuale.
/// Bug: il nemico usava sempre moves.first() → sempre la stessa mossa.
/// Fix: scegliere con Rng tra le mosse disponibili con PP > 0.

use game_core::battle::rng::Rng;
use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn tackle() -> Move {
    Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35)
}

fn ember() -> Move {
    Move::new("ember", Type::Fire, MoveCategory::Special, 40, 100, 25)
}

fn water_gun() -> Move {
    Move::new("water-gun", Type::Water, MoveCategory::Special, 40, 100, 25)
}

fn thunder_shock() -> Move {
    Move::new("thunder-shock", Type::Electric, MoveCategory::Special, 40, 100, 30)
}

fn foe_with_moves(moves: &[Move]) -> Pokemon {
    let mut p = Pokemon::new("Foe", Type::Normal, None, Stats::new(50, 50, 50, 50, 50, 50), 100, 10);
    for m in moves {
        p.add_move(m.clone());
    }
    p
}

/// Con una sola mossa, deve scegliere sempre quella.
#[test]
fn una_mossa_sceglie_sempre_la_prima() {
    let foe = foe_with_moves(&[tackle()]);
    for seed in 0..20u64 {
        let mut rng = Rng::new(seed);
        let chosen = game_core::battle::ai::choose_foe_move(&foe, &mut rng);
        assert_eq!(chosen.name, "tackle");
    }
}

/// Con più mosse, semi diversi producono scelte diverse (distribuzione non costante).
#[test]
fn quattro_mosse_non_sceglie_sempre_la_prima() {
    let foe = foe_with_moves(&[tackle(), ember(), water_gun(), thunder_shock()]);
    let mut counts = [0u32; 4];
    for seed in 0..200u64 {
        let mut rng = Rng::new(seed * 7 + 13);
        let chosen = game_core::battle::ai::choose_foe_move(&foe, &mut rng);
        let idx = foe.moves.iter().position(|m| m.name == chosen.name).unwrap();
        counts[idx] += 1;
    }
    // Ogni mossa deve essere scelta almeno una volta in 200 campioni
    for (i, &c) in counts.iter().enumerate() {
        assert!(c > 0, "mossa {i} non è mai stata scelta in 200 campioni");
    }
    // La prima mossa non deve dominare (< 70% delle scelte)
    assert!(counts[0] < 140, "la prima mossa viene scelta {}% delle volte, atteso < 70%", counts[0] / 2);
}

/// Se una mossa ha PP = 0, non deve essere scelta.
#[test]
fn mossa_senza_pp_non_viene_scelta() {
    let mut m1 = tackle();
    m1.current_pp = 0; // PP esauriti
    let foe = foe_with_moves(&[m1, ember()]);
    for seed in 0..50u64 {
        let mut rng = Rng::new(seed);
        let chosen = game_core::battle::ai::choose_foe_move(&foe, &mut rng);
        assert_eq!(chosen.name, "ember", "tackle ha PP=0, deve essere scelta ember");
    }
}

/// Se tutte le mosse hanno PP = 0, deve restituire un fallback (Struggle).
#[test]
fn tutte_le_mosse_senza_pp_usa_struggle() {
    let mut m1 = tackle();
    let mut m2 = ember();
    m1.current_pp = 0;
    m2.current_pp = 0;
    let foe = foe_with_moves(&[m1, m2]);
    let mut rng = Rng::new(42);
    let chosen = game_core::battle::ai::choose_foe_move(&foe, &mut rng);
    assert_eq!(chosen.name, "struggle", "con tutti PP=0 deve usare struggle");
}

/// La distribuzione con 2 mosse deve essere circa 50/50 su molti campioni.
#[test]
fn due_mosse_distribuzione_approssimata() {
    let foe = foe_with_moves(&[tackle(), ember()]);
    let mut count_tackle = 0u32;
    let mut count_ember = 0u32;
    for seed in 0..400u64 {
        let mut rng = Rng::new(seed * 3 + 7);
        let chosen = game_core::battle::ai::choose_foe_move(&foe, &mut rng);
        if chosen.name == "tackle" { count_tackle += 1; } else { count_ember += 1; }
    }
    // Con 400 campioni, ogni mossa deve avere tra 30% e 70% (range generoso per LCG)
    assert!(count_tackle > 100 && count_tackle < 300,
        "tackle scelto {count_tackle} volte su 400, atteso 30-70%");
    assert!(count_ember > 100 && count_ember < 300,
        "ember scelta {count_ember} volte su 400, atteso 30-70%");
}
