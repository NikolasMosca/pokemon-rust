#![allow(dead_code)]

use game_core::battle::rng::Rng;
use game_core::moves::{Move, MoveCategory, MoveEffect};
use game_core::moves_db::pick_moves;
use game_core::pokemon::{Pokemon, Stats};
use game_core::types::Type;
use crate::core::api::{ChainLink, MoveDetail, PokemonData};
use crate::core::cache::PokemonCache;

// Delay UI tra le fasi del turno in millisecondi.
pub const TURN_DELAY_MS: u32 = 800;

// --- Pool selvatici ---

struct PoolEntry {
    species: &'static str,
    weight: u32,
}

const WILD_POOL: &[PoolEntry] = &[
    // Comuni (30)
    PoolEntry { species: "rattata",   weight: 30 },
    PoolEntry { species: "pidgey",    weight: 30 },
    PoolEntry { species: "caterpie",  weight: 30 },
    PoolEntry { species: "weedle",    weight: 30 },
    PoolEntry { species: "geodude",   weight: 30 },
    // Non comuni (15)
    PoolEntry { species: "ekans",     weight: 15 },
    PoolEntry { species: "sandshrew", weight: 15 },
    PoolEntry { species: "jigglypuff",weight: 15 },
    PoolEntry { species: "oddish",    weight: 15 },
    PoolEntry { species: "psyduck",   weight: 15 },
    // Rari (7)
    PoolEntry { species: "growlithe", weight: 7 },
    PoolEntry { species: "abra",      weight: 7 },
    PoolEntry { species: "machop",    weight: 7 },
    PoolEntry { species: "gastly",    weight: 7 },
    PoolEntry { species: "onix",      weight: 7 },
    // Molto rari (3)
    PoolEntry { species: "scyther",   weight: 3 },
    PoolEntry { species: "lapras",    weight: 3 },
    PoolEntry { species: "dratini",   weight: 3 },
    PoolEntry { species: "eevee",     weight: 3 },
    PoolEntry { species: "porygon",   weight: 3 },
];

const TOTAL_WEIGHT: u32 = 30 * 5 + 15 * 5 + 7 * 5 + 3 * 5; // 300

/// Sceglie una specie dal pool pesato usando Rng.
pub fn pick_wild_species(rng: &mut Rng) -> &'static str {
    let roll = rng.next(TOTAL_WEIGHT as u64) as u32;
    let mut acc = 0u32;
    for entry in WILD_POOL {
        acc += entry.weight;
        if roll < acc {
            return entry.species;
        }
    }
    WILD_POOL.last().unwrap().species
}

/// Calcola il livello del nemico: `avg ± 2`, minimo 3.
pub fn enemy_level(avg: u8, rng: &mut Rng) -> u8 {
    let delta = rng.next(5) as i32 - 2; // -2..=+2
    ((avg as i32 + delta).max(3).min(100)) as u8
}

// --- Evoluzione ---

/// Percorre la chain e restituisce il nome della specie corretta per `level`.
/// Ritorna l'ultima forma raggiungibile al livello dato.
pub fn resolve_evolution(chain: &ChainLink, level: u8) -> String {
    // Controlla se possiamo evolvere verso la forma successiva
    for next in &chain.evolves_to {
        let min_level = next.evolution_details
            .iter()
            .filter_map(|d| d.min_level)
            .next();

        if let Some(min) = min_level {
            if level >= min {
                return resolve_evolution(next, level);
            }
        }
    }
    chain.species.name.clone()
}

// --- Mosse ---

const LEVEL_UP_METHOD: &str = "level-up";

/// Estrae dal PokemonData le mosse apprese tramite level-up fino al livello dato,
/// ordinate per livello decrescente (ultime apprese prima). Max 4.
pub fn pick_moves_for_level(data: &PokemonData, level: u8) -> Vec<String> {
    let mut learned: Vec<(u8, String)> = data.moves.iter()
        .filter_map(|entry| {
            let detail = entry.version_group_details.iter()
                .filter(|d| d.move_learn_method.name == LEVEL_UP_METHOD)
                .max_by_key(|d| d.level_learned_at)?;
            if detail.level_learned_at <= level && detail.level_learned_at > 0 {
                Some((detail.level_learned_at, entry.move_data.name.clone()))
            } else {
                None
            }
        })
        .collect();

    learned.sort_by(|a, b| b.0.cmp(&a.0));
    learned.dedup_by(|a, b| a.1 == b.1);
    learned.into_iter().take(4).map(|(_, name)| name).collect()
}

// --- Costruzione Pokemon da API data ---

pub fn type_from_str(s: &str) -> Type {
    match s {
        "normal"   => Type::Normal,
        "fire"     => Type::Fire,
        "water"    => Type::Water,
        "electric" => Type::Electric,
        "grass"    => Type::Grass,
        "ice"      => Type::Ice,
        "fighting" => Type::Fighting,
        "poison"   => Type::Poison,
        "ground"   => Type::Ground,
        "flying"   => Type::Flying,
        "psychic"  => Type::Psychic,
        "bug"      => Type::Bug,
        "rock"     => Type::Rock,
        "ghost"    => Type::Ghost,
        "dragon"   => Type::Dragon,
        "dark"     => Type::Dark,
        "steel"    => Type::Steel,
        "fairy"    => Type::Fairy,
        _ => Type::Normal,
    }
}

pub fn category_from_str(s: &str) -> MoveCategory {
    match s {
        "physical" => MoveCategory::Physical,
        "special"  => MoveCategory::Special,
        _ => MoveCategory::Status,
    }
}

/// Costruisce una `Move` da un `MoveDetail` API, applicando l'effetto corretto se riconosciuto.
pub fn move_from_detail(md: &MoveDetail) -> Move {
    let category = category_from_str(&md.damage_class.name);
    let power    = md.power.unwrap_or(0) as u8;
    let accuracy = md.accuracy.unwrap_or(100) as u8;
    let pp       = md.pp.unwrap_or(10) as u8;
    let move_type = type_from_str(&md.move_type.name);
    // Safety: Box::leak — i nomi vivono per tutta la durata del programma WASM.
    let name: &'static str = Box::leak(md.name.clone().into_boxed_str());

    let effect = match md.name.as_str() {
        "recover" | "soft-boiled" | "milk-drink" | "moonlight" | "morning-sun" | "synthesis"
            => MoveEffect::Heal { percent: 50 },
        "slack-off" | "roost"
            => MoveEffect::Heal { percent: 50 },
        "absorb" | "mega-drain" | "giga-drain" | "drain-punch" | "horn-leech" | "leech-life"
            => MoveEffect::Drain { percent: 50 },
        _ => MoveEffect::None,
    };

    Move::new(name, move_type, category, power, accuracy, pp).with_effect(effect)
}

pub fn build_pokemon(data: &PokemonData, level: u8, move_details: &[MoveDetail]) -> Pokemon {
    let primary_type = data.types.iter()
        .find(|t| t.slot == 1)
        .map(|t| type_from_str(&t.type_data.name))
        .unwrap_or(Type::Normal);

    let secondary_type = data.types.iter()
        .find(|t| t.slot == 2)
        .map(|t| type_from_str(&t.type_data.name));

    let get_stat = |name: &str| -> u32 {
        data.stats.iter()
            .find(|s| s.stat.name == name)
            .map(|s| s.base_stat)
            .unwrap_or(50)
    };

    let stats = Stats::new(
        get_stat("hp"),
        get_stat("attack"),
        get_stat("defense"),
        get_stat("special-attack"),
        get_stat("special-defense"),
        get_stat("speed"),
    );

    let base_exp = data.base_experience.unwrap_or(50);
    let mut pokemon = Pokemon::new(&data.name, primary_type, secondary_type, stats, base_exp, level);
    pokemon.pokedex_id = Some(data.id);

    for md in move_details {
        let m = move_from_detail(md);
        if m.is_supported() {
            pokemon.add_move(m);
        }
    }

    pokemon
}

/// Assegna esattamente 4 mosse al Pokémon usando il DB hardcoded.
/// Garantisce sempre 4 mosse, nessun fetch API necessario.
pub fn assign_moves(pokemon: &mut Pokemon, level: u8, rng: &mut Rng) {
    let moves = pick_moves(pokemon.primary_type, level, rng);
    for m in moves {
        pokemon.add_move(m);
    }
}

/// Genera un Pokémon nemico completo: specie, livello, forma evoluta, mosse.
/// Richiede accesso asincrono alla cache.
pub async fn generate_wild_pokemon(
    avg_level: u8,
    rng: &mut Rng,
    cache: &PokemonCache,
) -> Result<Pokemon, String> {
    let base_species = pick_wild_species(rng);
    let level = enemy_level(avg_level, rng);

    // Risolve evoluzione
    let species = cache.fetch_species(base_species).await?;
    let evo_chain = cache.fetch_evo_chain(&species.evolution_chain.url).await?;
    let final_species = resolve_evolution(&evo_chain.chain, level);

    // Fetch dati Pokémon nella forma corretta
    let data = cache.fetch(&final_species).await?;

    let mut pokemon = build_pokemon(&data, level, &[]);
    assign_moves(&mut pokemon, level, rng);

    Ok(pokemon)
}

/// Genera un team nemico per un allenatore (N Pokémon = team_size del giocatore).
pub async fn generate_trainer_team(
    avg_level: u8,
    team_size: usize,
    rng: &mut Rng,
    cache: &PokemonCache,
) -> Result<Vec<Pokemon>, String> {
    let mut team = Vec::new();
    for _ in 0..team_size.max(1) {
        let p = generate_wild_pokemon(avg_level, rng, cache).await?;
        team.push(p);
    }
    Ok(team)
}

/// Genera il team del capopalestra (team_size + 1, livello avg + 3).
pub async fn generate_gym_leader_team(
    avg_level: u8,
    team_size: usize,
    rng: &mut Rng,
    cache: &PokemonCache,
) -> Result<Vec<Pokemon>, String> {
    let leader_level = (avg_level as u32 + 3).min(100) as u8;
    generate_trainer_team(leader_level, team_size + 1, rng, cache).await
}

/// Lista degli starter disponibili per la scelta iniziale.
pub const STARTER_POOL: &[&str] = &[
    "bulbasaur", "charmander", "squirtle",
    "pikachu", "eevee",
    "chikorita", "cyndaquil", "totodile",
    "treecko", "torchic", "mudkip",
];

/// Sceglie 3 starter unici random dal pool.
pub fn pick_starters(rng: &mut Rng) -> [&'static str; 3] {
    let n = STARTER_POOL.len() as u64;
    let i0 = rng.next(n) as usize;
    let mut i1 = rng.next(n) as usize;
    while i1 == i0 { i1 = rng.next(n) as usize; }
    let mut i2 = rng.next(n) as usize;
    while i2 == i0 || i2 == i1 { i2 = rng.next(n) as usize; }
    [STARTER_POOL[i0], STARTER_POOL[i1], STARTER_POOL[i2]]
}
