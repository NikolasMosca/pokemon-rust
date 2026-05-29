use serde::{Deserialize, Serialize};
use crate::core::api::PokemonData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub money: u32,
    pub playtime_seconds: u64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            name: "Player".to_string(),
            money: 3000,
            playtime_seconds: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunProgress {
    pub badges: Vec<String>,
    pub current_route: String,
    pub step: u32,
}

impl Default for RunProgress {
    fn default() -> Self {
        Self {
            badges: vec![],
            current_route: "pallet-town".to_string(),
            step: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEntry {
    pub slot: u8,
    pub species: String,
    pub level: u8,
    pub current_hp: u32,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPokemon {
    pub name: String,
    pub data: PokemonData,
}
