#![allow(dead_code)]

use std::collections::HashMap;
use leptos::prelude::*;
use crate::core::api::{
    get_evolution_chain, get_move, get_pokemon, get_species,
    EvolutionChain, MoveDetail, PokemonData, PokemonSpecies,
};
use crate::core::storage::use_store;

#[derive(Clone)]
pub struct PokemonCache {
    pokemon: RwSignal<HashMap<String, PokemonData>>,
    moves: RwSignal<HashMap<String, MoveDetail>>,
    species: RwSignal<HashMap<String, PokemonSpecies>>,
    evo_chains: RwSignal<HashMap<String, EvolutionChain>>,
}

impl PokemonCache {
    pub fn new() -> Self {
        Self {
            pokemon: RwSignal::new(HashMap::new()),
            moves: RwSignal::new(HashMap::new()),
            species: RwSignal::new(HashMap::new()),
            evo_chains: RwSignal::new(HashMap::new()),
        }
    }

    // --- Pokemon ---

    pub fn get(&self, name: &str) -> Option<PokemonData> {
        self.pokemon.read().get(name).cloned()
    }

    pub async fn fetch(&self, name: &str) -> Result<PokemonData, String> {
        let key = name.to_lowercase();
        if let Some(cached) = self.get(&key) {
            return Ok(cached);
        }
        if let Some(store) = use_store() {
            if let Ok(Some(data)) = store.load_cached_pokemon(&key).await {
                self.pokemon.write().insert(key.clone(), data.clone());
                return Ok(data);
            }
        }
        let data = get_pokemon(&key).await?;
        self.pokemon.write().insert(key.clone(), data.clone());
        persist_pokemon_bg(key, data.clone());
        Ok(data)
    }

    // --- Move ---

    pub fn get_move(&self, name: &str) -> Option<MoveDetail> {
        self.moves.read().get(name).cloned()
    }

    pub async fn fetch_move(&self, name: &str) -> Result<MoveDetail, String> {
        let key = name.to_lowercase();
        if let Some(cached) = self.get_move(&key) {
            return Ok(cached);
        }
        let data = get_move(&key).await?;
        self.moves.write().insert(key, data.clone());
        Ok(data)
    }

    // --- Species ---

    pub fn get_species(&self, name: &str) -> Option<PokemonSpecies> {
        self.species.read().get(name).cloned()
    }

    pub async fn fetch_species(&self, name: &str) -> Result<PokemonSpecies, String> {
        let key = name.to_lowercase();
        if let Some(cached) = self.get_species(&key) {
            return Ok(cached);
        }
        let data = get_species(&key).await?;
        self.species.write().insert(key, data.clone());
        Ok(data)
    }

    // --- Evolution Chain ---

    pub fn get_evo_chain(&self, url: &str) -> Option<EvolutionChain> {
        self.evo_chains.read().get(url).cloned()
    }

    pub async fn fetch_evo_chain(&self, url: &str) -> Result<EvolutionChain, String> {
        if let Some(cached) = self.get_evo_chain(url) {
            return Ok(cached);
        }
        let data = get_evolution_chain(url).await?;
        self.evo_chains.write().insert(url.to_string(), data.clone());
        Ok(data)
    }
}

fn persist_pokemon_bg(name: String, data: PokemonData) {
    if let Some(store) = use_store() {
        leptos::task::spawn_local(async move {
            if let Err(e) = store.cache_pokemon(&name, &data).await {
                leptos::logging::warn!("Cache IDB fallita per {}: {}", name, e);
            }
        });
    }
}

pub fn provide_pokemon_cache() {
    provide_context(PokemonCache::new());
}

pub fn use_pokemon_cache() -> PokemonCache {
    use_context::<PokemonCache>().expect("PokemonCache non trovata nel contesto")
}
