use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use zeroize::Zeroizing;

use crate::error::CirrusError;

const STORE_PATH: &str = "cirrus.bin";
const KEY_REFRESH_TOKEN: &str = "msa_refresh_token";
const KEY_CLIENT_ID: &str = "msa_client_id";

pub fn save_refresh_token(app: &AppHandle, token: &str) -> Result<(), CirrusError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| CirrusError::Store(e.to_string()))?;
    store.set(KEY_REFRESH_TOKEN, serde_json::Value::String(token.to_string()));
    store.save().map_err(|e| CirrusError::Store(e.to_string()))?;
    Ok(())
}

pub fn load_refresh_token(app: &AppHandle) -> Result<Option<Zeroizing<String>>, CirrusError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| CirrusError::Store(e.to_string()))?;
    match store.get(KEY_REFRESH_TOKEN) {
        Some(serde_json::Value::String(s)) => Ok(Some(Zeroizing::new(s))),
        _ => Ok(None),
    }
}

pub fn delete_refresh_token(app: &AppHandle) -> Result<(), CirrusError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| CirrusError::Store(e.to_string()))?;
    store.delete(KEY_REFRESH_TOKEN);
    store.save().map_err(|e| CirrusError::Store(e.to_string()))?;
    Ok(())
}

pub fn save_client_id(app: &AppHandle, client_id: &str) -> Result<(), CirrusError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| CirrusError::Store(e.to_string()))?;
    store.set(KEY_CLIENT_ID, serde_json::Value::String(client_id.to_string()));
    store.save().map_err(|e| CirrusError::Store(e.to_string()))?;
    Ok(())
}

pub fn load_client_id(app: &AppHandle) -> Result<Option<String>, CirrusError> {
    let store = app
        .store(STORE_PATH)
        .map_err(|e| CirrusError::Store(e.to_string()))?;
    match store.get(KEY_CLIENT_ID) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Ok(Some(s)),
        _ => Ok(None),
    }
}
