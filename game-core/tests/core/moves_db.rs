/// Test TDD per moves_db: database hardcoded di mosse e funzione di selezione.
///
/// Invarianti da garantire:
/// 1. Ogni tipo ha almeno N mosse nel DB
/// 2. pick_moves restituisce sempre esattamente 4 mosse
/// 3. Nessuna mossa duplicata nel risultato
/// 4. Tutte le mosse restituite sono supportate (deals_damage o Heal/Drain)
/// 5. La prima mossa è sempre dello stesso tipo del Pokémon (STAB garantito)
/// 6. Il risultato è deterministico per stesso seed RNG
/// 7. Funziona per livelli bassi (1-10), medi (20-40) e alti (50+)

use game_core::moves_db::{pick_moves, ALL_MOVES};
use game_core::battle::rng::Rng;
use game_core::types::Type;

const ALL_TYPES: &[Type] = &[
    Type::Normal, Type::Fire, Type::Water, Type::Electric, Type::Grass,
    Type::Ice, Type::Fighting, Type::Poison, Type::Ground, Type::Flying,
    Type::Psychic, Type::Bug, Type::Rock, Type::Ghost, Type::Dragon,
    Type::Dark, Type::Steel, Type::Fairy,
];

// ── Invarianti sul DB ─────────────────────────────────────────────────────────

/// Il DB non deve essere vuoto.
#[test]
fn db_non_vuoto() {
    assert!(!ALL_MOVES.is_empty(), "ALL_MOVES non deve essere vuoto");
}

/// Ogni tipo deve avere almeno 4 mosse nel DB (altrimenti pick_moves non può
/// garantire 4 mosse diverse dello stesso tipo).
#[test]
fn ogni_tipo_ha_almeno_4_mosse() {
    for t in ALL_TYPES {
        let count = ALL_MOVES.iter().filter(|m| m.move_type == *t).count();
        assert!(
            count >= 4,
            "il tipo {:?} ha solo {} mosse nel DB — servono almeno 4",
            t, count
        );
    }
}

/// Tutte le mosse nel DB devono essere supportate (danno diretto o Heal/Drain).
#[test]
fn tutte_le_mosse_db_sono_supportate() {
    for m in ALL_MOVES {
        assert!(
            m.is_supported(),
            "la mossa '{}' nel DB non è supportata (power=0 e nessun effetto Heal/Drain)",
            m.name
        );
    }
}

/// Nessun duplicato di nome nel DB.
#[test]
fn nessun_duplicato_nel_db() {
    let mut names: Vec<&str> = ALL_MOVES.iter().map(|m| m.name).collect();
    names.sort();
    let before = names.len();
    names.dedup();
    assert_eq!(
        names.len(), before,
        "ci sono mosse duplicate nel DB"
    );
}

/// Tutte le mosse hanno power > 0 oppure sono Heal/Drain.
#[test]
fn power_coerente_con_categoria() {
    use game_core::moves::{MoveCategory, MoveEffect};
    for m in ALL_MOVES {
        if m.category == MoveCategory::Status {
            // Solo le Status con effetto Heal sono ammesse
            assert!(
                matches!(m.effect, MoveEffect::Heal { .. } | MoveEffect::Drain { .. }),
                "mossa Status '{}' nel DB senza Heal/Drain non è ammessa",
                m.name
            );
        } else {
            assert!(
                m.power > 0,
                "mossa Physical/Special '{}' nel DB ha power=0",
                m.name
            );
        }
    }
}

// ── Invarianti su pick_moves ──────────────────────────────────────────────────

/// pick_moves deve restituire esattamente 4 mosse per qualsiasi tipo e livello.
#[test]
fn pick_moves_restituisce_sempre_4() {
    let mut rng = Rng::new(42);
    for t in ALL_TYPES {
        for level in [1u8, 5, 10, 20, 30, 50, 80, 100] {
            let moves = pick_moves(*t, level, &mut rng);
            assert_eq!(
                moves.len(), 4,
                "pick_moves({:?}, {}) ha restituito {} mosse invece di 4",
                t, level, moves.len()
            );
        }
    }
}

/// Nessuna mossa duplicata nel risultato di pick_moves.
#[test]
fn pick_moves_nessun_duplicato() {
    let mut rng = Rng::new(99);
    for t in ALL_TYPES {
        let moves = pick_moves(*t, 20, &mut rng);
        let mut names: Vec<&str> = moves.iter().map(|m| m.name).collect();
        names.sort();
        let before = names.len();
        names.dedup();
        assert_eq!(
            names.len(), before,
            "pick_moves({:?}, 20) ha restituito mosse duplicate: {:?}",
            t, moves.iter().map(|m| m.name).collect::<Vec<_>>()
        );
    }
}

/// Tutte le mosse restituite da pick_moves devono essere supportate.
#[test]
fn pick_moves_tutte_supportate() {
    let mut rng = Rng::new(7);
    for t in ALL_TYPES {
        let moves = pick_moves(*t, 15, &mut rng);
        for m in &moves {
            assert!(
                m.is_supported(),
                "pick_moves({:?}) ha restituito '{}' che non è supportata",
                t, m.name
            );
        }
    }
}

/// La prima mossa deve essere di tipo STAB (stesso tipo del Pokémon) per garantire
/// che ogni Pokémon abbia almeno una mossa offensiva del suo tipo.
#[test]
fn pick_moves_prima_mossa_stab() {
    let mut rng = Rng::new(1);
    for t in ALL_TYPES {
        let moves = pick_moves(*t, 10, &mut rng);
        let first = &moves[0];
        assert_eq!(
            first.move_type, *t,
            "la prima mossa di pick_moves({:?}) è di tipo {:?} — deve essere STAB",
            t, first.move_type
        );
    }
}

/// Il risultato è deterministico: stesso tipo + livello + seed → stesso output.
#[test]
fn pick_moves_deterministico() {
    let moves_a = pick_moves(Type::Fire, 20, &mut Rng::new(42));
    let moves_b = pick_moves(Type::Fire, 20, &mut Rng::new(42));
    let names_a: Vec<&str> = moves_a.iter().map(|m| m.name).collect();
    let names_b: Vec<&str> = moves_b.iter().map(|m| m.name).collect();
    assert_eq!(names_a, names_b, "pick_moves deve essere deterministico con lo stesso seed");
}

/// Pokémon a livello basso (1-5) ottengono comunque 4 mosse valide.
#[test]
fn pick_moves_livello_basso_garantisce_4() {
    let mut rng = Rng::new(123);
    for t in ALL_TYPES {
        let moves = pick_moves(*t, 1, &mut rng);
        assert_eq!(moves.len(), 4,
            "anche a livello 1, {:?} deve avere 4 mosse", t);
        assert!(
            moves.iter().any(|m| m.deals_damage()),
            "a livello 1, {:?} deve avere almeno una mossa che infligge danno", t
        );
    }
}

/// Pokémon ad alto livello ricevono mosse mediamente più potenti di quelli a livello basso.
/// Questo verifica che la selezione scala con il livello.
#[test]
fn pick_moves_scala_con_livello() {
    let avg_power = |moves: &[game_core::moves::Move]| -> u32 {
        let dmg: Vec<_> = moves.iter().filter(|m| m.power > 0).collect();
        if dmg.is_empty() { return 0; }
        dmg.iter().map(|m| m.power as u32).sum::<u32>() / dmg.len() as u32
    };

    let mut rng_low  = Rng::new(42);
    let mut rng_high = Rng::new(42);

    let low_moves  = pick_moves(Type::Fire, 5,  &mut rng_low);
    let high_moves = pick_moves(Type::Fire, 60, &mut rng_high);

    let low_avg  = avg_power(&low_moves);
    let high_avg = avg_power(&high_moves);

    assert!(
        high_avg >= low_avg,
        "le mosse a livello alto (avg power={}) devono essere >= di quelle a livello basso (avg power={})",
        high_avg, low_avg
    );
}

/// pick_moves con tipo Normal deve restituire 4 mosse valide
/// (Normal ha il pool più ampio — test di sanity base).
#[test]
fn pick_moves_normal_type() {
    let mut rng = Rng::new(0);
    let moves = pick_moves(Type::Normal, 10, &mut rng);
    assert_eq!(moves.len(), 4);
    assert!(moves.iter().all(|m| m.is_supported()));
}

/// pick_moves deve restituire mosse che infliggono danno — nessun set di sole Heal.
#[test]
fn pick_moves_ha_sempre_mossa_danno() {
    let mut rng = Rng::new(55);
    for t in ALL_TYPES {
        let moves = pick_moves(*t, 25, &mut rng);
        assert!(
            moves.iter().any(|m| m.deals_damage()),
            "pick_moves({:?}) non ha nessuna mossa che infligge danno",
            t
        );
    }
}
