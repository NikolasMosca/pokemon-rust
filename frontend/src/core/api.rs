#![allow(dead_code)]

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonData {
    pub id: u32,
    pub name: String,
    pub base_experience: Option<u32>,
    pub height: u32,
    pub weight: u32,
    pub sprites: Sprites,
    pub stats: Vec<StatEntry>,
    pub moves: Vec<MoveEntry>,
    pub types: Vec<TypeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprites {
    pub front_default: Option<String>,
    pub back_default: Option<String>,
    pub front_shiny: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatEntry {
    pub base_stat: u32,
    pub stat: NamedResource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveEntry {
    #[serde(rename = "move")]
    pub move_data: NamedResource,
    pub version_group_details: Vec<MoveVersionDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveVersionDetail {
    pub level_learned_at: u8,
    pub move_learn_method: NamedResource,
    pub version_group: NamedResource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEntry {
    pub slot: u8,
    #[serde(rename = "type")]
    pub type_data: NamedResource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedResource {
    pub name: String,
    pub url: String,
}

// --- Move detail ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveDetail {
    pub id: u32,
    pub name: String,
    pub power: Option<u32>,
    pub accuracy: Option<u32>,
    pub pp: Option<u32>,
    pub damage_class: NamedResource,
    #[serde(rename = "type")]
    pub move_type: NamedResource,
}

// --- Pokemon Species (per evolution chain url) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSpecies {
    pub name: String,
    pub evolution_chain: EvolutionChainRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionChainRef {
    pub url: String,
}

// --- Evolution Chain ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionChain {
    pub id: u32,
    pub chain: ChainLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainLink {
    pub species: NamedResource,
    pub evolution_details: Vec<EvolutionDetail>,
    pub evolves_to: Vec<ChainLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionDetail {
    pub min_level: Option<u8>,
}

// --- Fetch functions ---

pub async fn get_pokemon(name: &str) -> Result<PokemonData, String> {
    fetch_json(&format!("https://pokeapi.co/api/v2/pokemon/{}", name.to_lowercase())).await
}

pub async fn get_move(name: &str) -> Result<MoveDetail, String> {
    fetch_json(&format!("https://pokeapi.co/api/v2/move/{}", name.to_lowercase())).await
}

pub async fn get_species(name: &str) -> Result<PokemonSpecies, String> {
    fetch_json(&format!("https://pokeapi.co/api/v2/pokemon-species/{}", name.to_lowercase())).await
}

pub async fn get_evolution_chain(url: &str) -> Result<EvolutionChain, String> {
    fetch_json(url).await
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Errore rete: {e}"))?;

    if !response.ok() {
        return Err(format!("Risposta non ok: {} (status {})", url, response.status()));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Errore parsing JSON da {url}: {e}"))
}
