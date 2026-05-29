pub mod idb;
pub mod models;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Serialize};
use idb::{IdbBackend, STORE_API_CACHE, STORE_PLAYER, STORE_RUN, STORE_TEAM};
use models::{CachedPokemon, PlayerState, RunProgress, TeamEntry};
use crate::core::api::PokemonData;

/// SendWrapper rende Store Send+Sync per provide_context di Leptos 0.7.
/// Sicuro perché siamo single-threaded in WASM.
#[derive(Clone)]
pub struct Store(SendWrapper<IdbBackend>);

impl Store {
    fn new(backend: IdbBackend) -> Self {
        Self(SendWrapper::new(backend))
    }

    async fn save<T: Serialize>(&self, store: &str, key: &str, value: &T) -> Result<(), String> {
        self.0.save(store, key, value).await
    }

    async fn load<T: DeserializeOwned>(&self, store: &str, key: &str) -> Result<Option<T>, String> {
        self.0.load(store, key).await
    }

    pub async fn delete(&self, store: &str, key: &str) -> Result<(), String> {
        self.0.delete(store, key).await
    }

    pub async fn save_player(&self, state: &PlayerState) -> Result<(), String> {
        self.save(STORE_PLAYER, "current", state).await
    }

    pub async fn load_player(&self) -> Result<Option<PlayerState>, String> {
        self.load(STORE_PLAYER, "current").await
    }

    pub async fn save_run(&self, progress: &RunProgress) -> Result<(), String> {
        self.save(STORE_RUN, "current", progress).await
    }

    pub async fn load_run(&self) -> Result<Option<RunProgress>, String> {
        self.load(STORE_RUN, "current").await
    }

    pub async fn save_team_slot(&self, entry: &TeamEntry) -> Result<(), String> {
        let key = entry.slot.to_string();
        self.save(STORE_TEAM, &key, entry).await
    }

    pub async fn load_team(&self) -> Result<Vec<TeamEntry>, String> {
        let mut team = Vec::new();
        for slot in 0u8..6 {
            let key = slot.to_string();
            if let Some(entry) = self.load::<TeamEntry>(STORE_TEAM, &key).await? {
                team.push(entry);
            }
        }
        Ok(team)
    }

    pub async fn cache_pokemon(&self, name: &str, data: &PokemonData) -> Result<(), String> {
        let entry = CachedPokemon { name: name.to_string(), data: data.clone() };
        self.save(STORE_API_CACHE, name, &entry).await
    }

    pub async fn load_cached_pokemon(&self, name: &str) -> Result<Option<PokemonData>, String> {
        let entry: Option<CachedPokemon> = self.load(STORE_API_CACHE, name).await?;
        Ok(entry.map(|e| e.data))
    }
}

pub async fn provide_store() {
    match IdbBackend::open().await {
        Ok(backend) => provide_context(Store::new(backend)),
        Err(e) => leptos::logging::error!("Storage non disponibile: {}", e),
    }
}

pub fn use_store() -> Option<Store> {
    use_context::<Store>()
}
