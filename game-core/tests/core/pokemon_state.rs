use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;

fn pikachu() -> Pokemon {
    Pokemon::new("Pikachu", Type::Electric, None, Stats::new(35, 55, 40, 50, 50, 90), 112, 50)
}

#[test]
fn hp_non_scende_sotto_zero() {
    let mut p = pikachu();
    p.take_damage(9999);
    assert_eq!(p.current_hp, 0);
}

#[test]
fn hp_non_supera_max() {
    let mut p = pikachu();
    p.heal(9999);
    assert_eq!(p.current_hp, p.max_hp());
}

#[test]
fn heal_parziale_corretto() {
    let mut p = pikachu();
    let max = p.max_hp();
    p.take_damage(30);
    p.heal(10);
    assert_eq!(p.current_hp, max - 20);
}

#[test]
fn is_fainted_solo_a_zero() {
    let mut p = pikachu();
    assert!(!p.is_fainted());
    p.take_damage(p.current_hp);
    assert!(p.is_fainted());
}

#[test]
fn hp_iniziali_uguali_a_max() {
    let p = pikachu();
    assert_eq!(p.current_hp, p.max_hp());
}
