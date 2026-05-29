/// Test che verificano che il danno calcolato sia realistico rispetto ai dati
/// delle API PokeAPI — usando stat base reali di Pokémon noti.
///
/// I dati di riferimento sono verificati su:
///   https://pokeapi.co/api/v2/pokemon/rattata
///   https://pokeapi.co/api/v2/pokemon/bulbasaur
///
/// Formula stat effettivi Gen III+:
///   HP  = floor((2*base + 31) * level / 100) + level + 10
///   Atk = floor((2*base + 31) * level / 100) + 5
///   Def = floor((2*base + 31) * level / 100) + 5
///
/// Formula danno Gen III+ usa gli STAT EFFETTIVI, non i base stat.

use game_core::battle::damage::calculate_damage;
use game_core::battle::rng::Rng;
use game_core::moves::{Move, MoveCategory};
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn rng_no_crit() -> Rng { Rng::new(1) } // seed che non produce crit

/// Calcola lo stat effettivo di attacco/difesa/speed a un dato livello.
/// Formula Gen III+: floor((2*base + 31) * level / 100) + 5
fn effective_stat(base: u32, level: u8) -> u32 {
    (2 * base + 31) * level as u32 / 100 + 5
}

/// Rattata base stats (da PokeAPI): hp=30, atk=56, def=35, spatk=25, spdef=35, spd=72
fn rattata(level: u8) -> Pokemon {
    Pokemon::new("rattata", Type::Normal, None,
        Stats::new(30, 56, 35, 25, 35, 72), 51, level)
}

/// Bulbasaur base stats (da PokeAPI): hp=45, atk=49, def=49, spatk=65, spdef=65, spd=45
fn bulbasaur(level: u8) -> Pokemon {
    Pokemon::new("bulbasaur", Type::Grass, Some(Type::Poison),
        Stats::new(45, 49, 49, 65, 65, 45), 64, level)
}

fn tackle() -> Move {
    Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35)
}

// ── Test 1: stat effettivi corretti ──────────────────────────────────────────

/// Gli HP di Rattata lv7 devono essere 23 (formula standard Gen III).
/// hp = (2*30+31)*7/100 + 7 + 10 = 91*7/100 + 17 = 6 + 17 = 23
#[test]
fn rattata_lv7_hp_corretto() {
    let r = rattata(7);
    assert_eq!(r.max_hp(), 23,
        "Rattata lv7 deve avere 23 HP (formula Gen III con base_hp=30)");
}

/// Gli HP di Bulbasaur lv5 devono essere 21 (formula standard Gen III).
/// hp = (2*45+31)*5/100 + 5 + 10 = 121*5/100 + 15 = 6 + 15 = 21
#[test]
fn bulbasaur_lv5_hp_corretto() {
    let b = bulbasaur(5);
    assert_eq!(b.max_hp(), 21,
        "Bulbasaur lv5 deve avere 21 HP (formula Gen III con base_hp=45)");
}

// ── Test 2: danno NON deve essere one-shot in condizioni normali ─────────────

/// Rattata lv7 con Tackle NON deve uccidere Bulbasaur lv5 in un colpo solo.
/// Con stat effettivi corretti:
///   atk_eff = (2*56+31)*7/100 + 5 = 143*7/100 + 5 = 10+5 = 15
///   def_eff = (2*49+31)*5/100 + 5 = 129*5/100 + 5 = 6+5  = 11
///   danno  = (2*7/5+2) * 40 * 15 / 11 / 50 + 2 = 4*40*15/11/50+2 = 2400/11/50+2 = 4+2 = 6
///   HP Bulbasaur lv5 = 21
///   => 6 << 21, nessun one-shot.
///
/// Con base stat (bug attuale):
///   atk = 56, def = 49
///   danno = (2*7/5+2) * 40 * 56 / 49 / 50 + 2 = 4*40*56/49/50+2 = 8960/49/50+2 = 3+2 = 5
///   Anche con base stat il danno è 5 — ma con STAB o super-efficace diventa pericoloso.
///
/// Il vero problema emerge con STAB + crit + supereffcace:
/// Verifichiamo che SENZA questi moltiplicatori il danno sia <= 50% degli HP del difensore.
#[test]
fn tackle_lv7_non_one_shot_lv5_senza_modificatori() {
    let attacker = rattata(7);
    let defender = bulbasaur(5);
    let defender_hp = defender.max_hp();

    let result = calculate_damage(&attacker, &defender, &tackle(), &mut rng_no_crit());

    // Tackle (Normal) vs Bulbasaur (Grass/Poison): nessun STAB, nessun tipo speciale
    assert!(!result.is_crit, "precondizione: il seed deve non produrre crit");
    assert_eq!(result.effectiveness, 1.0, "Tackle Normal vs Grass/Poison = 1.0x");

    // Il danno deve essere molto inferiore agli HP — non uno-shot
    assert!(result.damage < defender_hp,
        "Rattata lv7 Tackle NON deve fare one-shot su Bulbasaur lv5 (danno={}, HP={})",
        result.damage, defender_hp);

    // Deve servire almeno 3 turni per uccidere
    let turni_necessari = (defender_hp + result.damage - 1) / result.damage;
    assert!(turni_necessari >= 3,
        "Deve servire almeno 3 turni per uccidere (danno={}, HP={}, turni={})",
        result.damage, defender_hp, turni_necessari);
}

/// Stesso test dall'altro lato: Bulbasaur lv5 vs Rattata lv7 con Tackle.
#[test]
fn tackle_lv5_non_one_shot_lv7_senza_modificatori() {
    let attacker = bulbasaur(5);
    let defender = rattata(7);
    let defender_hp = defender.max_hp();

    let result = calculate_damage(&attacker, &defender, &tackle(), &mut rng_no_crit());

    assert!(!result.is_crit);
    assert!(result.damage < defender_hp,
        "Bulbasaur lv5 Tackle NON deve fare one-shot su Rattata lv7 (danno={}, HP={})",
        result.damage, defender_hp);

    let turni = (defender_hp + result.damage - 1) / result.damage;
    assert!(turni >= 3,
        "Deve servire almeno 3 turni (danno={}, HP={}, turni={})",
        result.damage, defender_hp, turni);
}

// ── Test 3: danno con stat effettivi vs base stat ────────────────────────────

/// Verifica che il danno calcolato sia coerente con la formula Gen III che usa
/// STAT EFFETTIVI calcolati per livello, non i base stat grezzi.
///
/// Con base stat grezzi (bug): atk=56 → danno sproporzionatamente alto per lv1-20.
/// Con stat effettivi (corretto): atk_eff(56,7)=15 → danno proporzionato al livello.
///
/// Questo test documenta i VALORI ATTESI con stat effettivi.
/// Se il test fallisce, significa che il codice usa ancora i base stat.
#[test]
fn danno_coerente_con_stat_effettivi_per_livello() {
    let attacker = rattata(7);  // atk base=56, atk_eff(56,7)=15
    let defender = bulbasaur(5); // def base=49, def_eff(49,5)=11

    // Valore atteso con STAT EFFETTIVI + STAB:
    // atk_eff = (2*56+31)*7/100 + 5 = 143*7/100+5 = 10+5 = 15
    // def_eff = (2*49+31)*5/100 + 5 = 129*5/100+5 = 6+5  = 11
    // base = (2*7/5+2)*40*15/11/50+2 = 4*600/11/50+2 = 4+2 = 6
    // STAB: Tackle Normal, Rattata Normal → ×1.5 → floor(6*1.5) = 9
    let expected_with_effective_stats = 9u32;

    // Valore con BASE STAT (comportamento buggy precedente):
    // base = 4*2240/49/50+2 = 5; con STAB = floor(5*1.5) = 7
    let expected_with_base_stats = 7u32;

    let result = calculate_damage(&attacker, &defender, &tackle(), &mut rng_no_crit());

    if result.damage == expected_with_base_stats {
        panic!(
            "Bug: il codice usa ancora base stat. danno={} (atteso {} con stat effettivi)",
            result.damage, expected_with_effective_stats
        );
    } else {
        assert_eq!(result.damage, expected_with_effective_stats,
            "Danno inatteso: {} (atteso {} con stat effettivi+STAB)",
            result.damage, expected_with_effective_stats);
    }
}

// ── Test 4: nessun one-shot a livelli bassi in nessuno scenario normale ──────

/// A livelli 5-10, con qualsiasi Pokémon delle prime generazioni,
/// un singolo attacco fisico senza crit e senza super-efficacia NON deve fare one-shot
/// su un Pokémon dello stesso livello con HP normali.
///
/// Machop (atk=80) vs Rattata (def=35, hp=30) — caso estremo per atk/def.
/// Mossa: Tackle (Normal) — nessun STAB su Machop Fighting, nessuna super-efficacia su Normal.
/// Con stat effettivi lv5:
///   atk_eff(80,5) = (2*80+31)*5/100+5 = 191*5/100+5 = 9+5 = 14
///   def_eff(35,5) = (2*35+31)*5/100+5 = 101*5/100+5 = 5+5 = 10
///   base = (2*5/5+2)*40*14/10/50+2 = 4*560/10/50+2 = 4+2 = 6
///   Rattata lv5 HP = 19 → nessun one-shot.
#[test]
fn nessun_one_shot_a_livello_basso_stesso_livello() {
    let mut machop = Pokemon::new("machop", Type::Fighting, None,
        Stats::new(70, 80, 50, 35, 35, 35), 61, 5);
    machop.add_move(Move::new("tackle", Type::Normal, MoveCategory::Physical, 40, 100, 35));

    let defender = rattata(5);
    let defender_hp = defender.max_hp();

    let result = calculate_damage(&machop, &defender, &machop.moves[0], &mut rng_no_crit());

    assert!(!result.is_crit, "precondizione: seed deve non produrre crit");
    assert_eq!(result.effectiveness, 1.0, "Tackle Normal vs Normal = 1.0x");
    assert!(result.damage < defender_hp,
        "Machop lv5 Tackle NON deve fare one-shot su Rattata lv5 (danno={}, HP={})\n\
         Se fallisce: il codice usa base stat invece di stat effettivi",
        result.damage, defender_hp);
}
