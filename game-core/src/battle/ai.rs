use crate::moves::Move;
use crate::pokemon::Pokemon;
use super::rng::Rng;

pub const STRUGGLE: Move = Move::new(
    "struggle",
    crate::types::Type::Normal,
    crate::moves::MoveCategory::Physical,
    50, 100, 1,
);

/// Sceglie una mossa casuale tra quelle disponibili (PP > 0) del Pokémon nemico.
/// Restituisce l'indice della mossa scelta, o None se tutte le PP sono esaurite (→ usa STRUGGLE).
pub fn choose_foe_move_idx(pokemon: &Pokemon, rng: &mut Rng) -> Option<usize> {
    let available: Vec<usize> = pokemon.moves.iter()
        .enumerate()
        .filter(|(_, m)| m.current_pp > 0)
        .map(|(i, _)| i)
        .collect();

    if available.is_empty() {
        return None;
    }

    let pick = rng.next(available.len() as u64) as usize;
    Some(available[pick])
}

/// Versione che restituisce un clone della mossa scelta (o STRUGGLE).
pub fn choose_foe_move(pokemon: &Pokemon, rng: &mut Rng) -> Move {
    match choose_foe_move_idx(pokemon, rng) {
        Some(idx) => pokemon.moves[idx].clone(),
        None => STRUGGLE,
    }
}
