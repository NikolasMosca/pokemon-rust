use std::sync::Arc;
use rexie::{ObjectStore, Rexie, TransactionMode};
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;

const DB_NAME: &str = "pokemon-rust";
const DB_VERSION: u32 = 1;

pub const STORE_PLAYER: &str = "player_state";
pub const STORE_RUN: &str = "run_progress";
pub const STORE_TEAM: &str = "team";
pub const STORE_API_CACHE: &str = "api_cache";

#[derive(Clone)]
pub struct IdbBackend {
    db: Arc<SendWrapper<Rexie>>,
}

impl IdbBackend {
    pub async fn open() -> Result<Self, String> {
        let db = Rexie::builder(DB_NAME)
            .version(DB_VERSION)
            .add_object_store(ObjectStore::new(STORE_PLAYER).auto_increment(false))
            .add_object_store(ObjectStore::new(STORE_RUN).auto_increment(false))
            .add_object_store(ObjectStore::new(STORE_TEAM).auto_increment(false))
            .add_object_store(ObjectStore::new(STORE_API_CACHE).auto_increment(false))
            .build()
            .await
            .map_err(|e| format!("IDB open error: {:?}", e))?;

        Ok(Self { db: Arc::new(SendWrapper::new(db)) })
    }

    pub async fn save<T: Serialize>(&self, store: &str, key: &str, value: &T) -> Result<(), String> {
        let js_value = serde_wasm_bindgen::to_value(value)
            .map_err(|e| format!("serialize error: {:?}", e))?;
        let js_key = JsValue::from_str(key);

        let tx = self.db
            .transaction(&[store], TransactionMode::ReadWrite)
            .map_err(|e| format!("tx error: {:?}", e))?;

        let obj_store = tx.store(store)
            .map_err(|e| format!("store error: {:?}", e))?;

        obj_store.put(&js_value, Some(&js_key))
            .await
            .map_err(|e| format!("put error: {:?}", e))?;

        tx.done().await.map_err(|e| format!("tx commit error: {:?}", e))?;
        Ok(())
    }

    pub async fn load<T: DeserializeOwned>(&self, store: &str, key: &str) -> Result<Option<T>, String> {
        let js_key = JsValue::from_str(key);

        let tx = self.db
            .transaction(&[store], TransactionMode::ReadOnly)
            .map_err(|e| format!("tx error: {:?}", e))?;

        let obj_store = tx.store(store)
            .map_err(|e| format!("store error: {:?}", e))?;

        let result = obj_store.get(js_key)
            .await
            .map_err(|e| format!("get error: {:?}", e))?;

        match result {
            None => Ok(None),
            Some(value) => {
                let deserialized = serde_wasm_bindgen::from_value(value)
                    .map_err(|e| format!("deserialize error: {:?}", e))?;
                Ok(Some(deserialized))
            }
        }
    }

    pub async fn delete(&self, store: &str, key: &str) -> Result<(), String> {
        let js_key = JsValue::from_str(key);

        let tx = self.db
            .transaction(&[store], TransactionMode::ReadWrite)
            .map_err(|e| format!("tx error: {:?}", e))?;

        let obj_store = tx.store(store)
            .map_err(|e| format!("store error: {:?}", e))?;

        obj_store.delete(js_key)
            .await
            .map_err(|e| format!("delete error: {:?}", e))?;

        tx.done().await.map_err(|e| format!("tx commit error: {:?}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::models::{PlayerState, RunProgress, TeamEntry};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn open_test_db() -> IdbBackend {
        IdbBackend::open().await.expect("IDB deve aprirsi")
    }

    #[wasm_bindgen_test]
    async fn player_state_save_load_round_trip() {
        let db = open_test_db().await;
        let state = PlayerState { name: "Ash".to_string(), money: 5000, playtime_seconds: 3600 };
        db.save(STORE_PLAYER, "current", &state).await.expect("save deve riuscire");
        let loaded: Option<PlayerState> = db.load(STORE_PLAYER, "current").await.expect("load deve riuscire");
        let loaded = loaded.expect("deve trovare il valore salvato");
        assert_eq!(loaded.name, "Ash");
        assert_eq!(loaded.money, 5000);
    }

    #[wasm_bindgen_test]
    async fn load_chiave_assente_restituisce_none() {
        let db = open_test_db().await;
        let result: Option<RunProgress> = db.load(STORE_RUN, "chiave-inesistente-xyz").await.expect("load deve riuscire");
        assert!(result.is_none());
    }

    #[wasm_bindgen_test]
    async fn delete_rimuove_il_valore() {
        let db = open_test_db().await;
        let entry = TeamEntry { slot: 0, species: "pikachu".to_string(), level: 25, current_hp: 45, nickname: None };
        db.save(STORE_TEAM, "0", &entry).await.expect("save deve riuscire");
        db.delete(STORE_TEAM, "0").await.expect("delete deve riuscire");
        let result: Option<TeamEntry> = db.load(STORE_TEAM, "0").await.expect("load deve riuscire");
        assert!(result.is_none());
    }

    #[wasm_bindgen_test]
    async fn run_progress_save_load_round_trip() {
        let db = open_test_db().await;
        let progress = RunProgress {
            badges: vec!["boulder".to_string()],
            current_route: "cerulean-city".to_string(),
            step: 42,
        };
        db.save(STORE_RUN, "current", &progress).await.expect("save deve riuscire");
        let loaded: Option<RunProgress> = db.load(STORE_RUN, "current").await.expect("load deve riuscire");
        let loaded = loaded.expect("deve trovare il valore salvato");
        assert_eq!(loaded.step, 42);
        assert_eq!(loaded.current_route, "cerulean-city");
    }
}
