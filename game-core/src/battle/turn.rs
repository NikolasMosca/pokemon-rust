use crate::battle::damage::{calculate_damage, DamageResult};
use crate::battle::rng::Rng;
use crate::moves::{Move, MoveEffect};
use crate::pokemon::Pokemon;

#[derive(Debug, Clone, PartialEq)]
pub enum TurnAction {
    UseMove(usize),
    Run,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurnResult {
    Ongoing,
    PlayerWon,
    EnemyWon,
    Fled,
}

#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub player_hit: DamageResult,
    pub enemy_hit: DamageResult,
    pub result: TurnResult,
    /// HP guadagnati dal player in questo turno (Heal o Drain).
    pub healed_player: u32,
    /// HP guadagnati dal nemico in questo turno (Heal o Drain).
    pub healed_enemy: u32,
}

/// Applica l'effetto di una mossa (Heal o Drain) al user dopo il calcolo danno.
/// Restituisce gli HP effettivamente guadagnati.
fn apply_move_effect(user: &mut Pokemon, m: &Move, dmg_dealt: u32) -> u32 {
    match m.effect {
        MoveEffect::None => 0,
        MoveEffect::Heal { percent } => {
            let amount = (user.max_hp() * percent as u32) / 100;
            let before = user.current_hp;
            user.heal(amount);
            user.current_hp - before
        }
        MoveEffect::Drain { percent } => {
            // dmg_dealt è già il danno realmente subito (take_damage lo clampa agli HP attuali).
            let amount = (dmg_dealt * percent as u32) / 100;
            let before = user.current_hp;
            user.heal(amount);
            user.current_hp - before
        }
    }
}

/// Esegue un turno completo con ordine per velocità e crit deterministico.
/// `rng` viene mutato in place — il chiamante lo conserva per il turno successivo.
pub fn execute_turn(
    player: &mut Pokemon,
    enemy: &mut Pokemon,
    player_action: TurnAction,
    enemy_move: &Move,
    rng: &mut Rng,
) -> TurnOutcome {
    if player_action == TurnAction::Run {
        return TurnOutcome {
            player_hit: DamageResult::zero(),
            enemy_hit: DamageResult::zero(),
            healed_player: 0,
            healed_enemy: 0,
            result: TurnResult::Fled,
        };
    }

    let TurnAction::UseMove(move_index) = player_action else {
        unreachable!()
    };

    let player_move = player.moves.get(move_index).cloned();
    let player_first = player.base_stats.speed >= enemy.base_stats.speed;

    if let Some(m) = player.moves.get_mut(move_index) {
        m.use_pp();
    }

    let mut player_hit = DamageResult::zero();
    let mut enemy_hit = DamageResult::zero();
    let mut healed_player: u32 = 0;
    let mut healed_enemy: u32 = 0;

    if player_first {
        if let Some(ref m) = player_move {
            let result = calculate_damage(player, enemy, m, rng);
            let actual = result.damage.min(enemy.current_hp);
            enemy.take_damage(result.damage);
            healed_player += apply_move_effect(player, m, actual);
            enemy_hit = result;
        }
        if !enemy.is_fainted() {
            let result = calculate_damage(enemy, player, enemy_move, rng);
            let actual = result.damage.min(player.current_hp);
            player.take_damage(result.damage);
            healed_enemy += apply_move_effect(enemy, enemy_move, actual);
            player_hit = result;
        }
    } else {
        let result = calculate_damage(enemy, player, enemy_move, rng);
        let actual = result.damage.min(player.current_hp);
        player.take_damage(result.damage);
        healed_enemy += apply_move_effect(enemy, enemy_move, actual);
        player_hit = result;
        if !player.is_fainted() {
            if let Some(ref m) = player_move {
                let result = calculate_damage(player, enemy, m, rng);
                let actual = result.damage.min(enemy.current_hp);
                enemy.take_damage(result.damage);
                healed_player += apply_move_effect(player, m, actual);
                enemy_hit = result;
            }
        }
    }

    let result = if player.is_fainted() {
        TurnResult::EnemyWon
    } else if enemy.is_fainted() {
        TurnResult::PlayerWon
    } else {
        TurnResult::Ongoing
    };

    TurnOutcome { player_hit, enemy_hit, healed_player, healed_enemy, result }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{Move, MoveCategory, MoveEffect};
    use crate::pokemon::{Pokemon, Stats};
    use crate::types::Type;
    use crate::battle::rng::Rng;

    fn poke(hp: u32, speed: u32) -> Pokemon {
        Pokemon::new(
            "Test",
            Type::Normal,
            None,
            Stats::new(hp, 100, 10, 10, 10, speed),
            100,
            5,
        )
    }

    fn lethal_move() -> Move {
        Move::new("Lethal", Type::Normal, MoveCategory::Physical, 255, 100, 10)
    }

    fn weak_move() -> Move {
        Move::new("Tap", Type::Normal, MoveCategory::Physical, 1, 100, 10)
    }

    fn rng() -> Rng { Rng::new(42) }

    /// Il nemico uccide il player con il suo attacco (player va per primo ma non uccide il foe).
    /// Il risultato deve essere EnemyWon, non Ongoing.
    #[test]
    fn enemy_kills_player_after_player_attacks_first() {
        let mut player = poke(10, 100); // player più veloce
        player.moves.push(weak_move());
        let mut enemy = poke(1000, 1);  // nemico lentissimo ma con tanti HP
        // Il nemico usa una mossa letale
        let foe_move = lethal_move();
        let mut r = rng();

        let out = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &foe_move, &mut r);

        assert_eq!(out.result, TurnResult::EnemyWon, "il player deve risultare sconfitto");
        assert!(player.is_fainted(), "il player deve essere a 0 HP");
        assert!(!enemy.is_fainted(), "il nemico deve essere ancora vivo");
    }

    /// Il player uccide il nemico quando va per primo — il nemico non deve contrattaccare.
    #[test]
    fn player_kills_enemy_first_no_counter() {
        let mut player = poke(1000, 100); // player più veloce e con tanti HP
        player.moves.push(lethal_move());
        let mut enemy = poke(10, 1);
        let foe_move = lethal_move();
        let mut r = rng();

        let out = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &foe_move, &mut r);

        assert_eq!(out.result, TurnResult::PlayerWon);
        assert_eq!(out.player_hit.damage, 0, "il foe non deve aver contrattaccato");
        assert!(!player.is_fainted());
    }

    /// Il nemico va per primo e uccide il player — il player non deve contrattaccare.
    #[test]
    fn enemy_kills_player_first_no_counter() {
        let mut player = poke(10, 1); // player più lento
        player.moves.push(lethal_move());
        let mut enemy = poke(1000, 100);
        let foe_move = lethal_move();
        let mut r = rng();

        let out = execute_turn(&mut player, &mut enemy, TurnAction::UseMove(0), &foe_move, &mut r);

        assert_eq!(out.result, TurnResult::EnemyWon);
        assert_eq!(out.enemy_hit.damage, 0, "il player non deve aver contrattaccato dopo essere svenuto");
        assert!(player.is_fainted());
    }
}
