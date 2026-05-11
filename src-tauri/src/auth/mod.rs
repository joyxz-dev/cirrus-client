pub mod microsoft;
pub mod minecraft;

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use zeroize::Zeroizing;

use crate::error::CirrusError;
use crate::store;
use microsoft::{DeviceCodeChallenge, DeviceCodeSession};

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub uuid: String,
    pub username: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: u64,
}

/// Internal session — holds the public Account plus the in-memory MC token.
/// The token never leaves the backend.
pub struct InternalSession {
    pub account: Account,
    pub mc_token: Zeroizing<String>,
}

pub type AccountState = Arc<Mutex<Option<InternalSession>>>;

pub fn new_account_state() -> AccountState {
    Arc::new(Mutex::new(None))
}

fn build_client() -> Result<reqwest::Client, CirrusError> {
    reqwest::Client::builder()
        .user_agent("Cirrus/0.1.0 (cirrusclient.gg)")
        .build()
        .map_err(Into::into)
}

fn resolve_client_id(app: &AppHandle) -> Result<String, CirrusError> {
    if let Some(id) = store::load_client_id(app)? {
        return Ok(id);
    }
    if let Some(id) = option_env!("AZURE_CLIENT_ID") {
        return Ok(id.to_string());
    }
    Err(CirrusError::Auth(
        "No Azure client ID configured. Go to Settings → Microsoft Auth to add one.".into(),
    ))
}

pub async fn start_auth(
    app: &AppHandle,
    state: &AccountState,
) -> Result<DeviceCodeChallenge, CirrusError> {
    let client = build_client()?;
    let client_id = resolve_client_id(app)?;

    let session = microsoft::begin_device_code_flow(&client, &client_id).await?;
    let challenge = session.challenge.clone();

    let app = app.clone();
    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        match finish_auth(&app, &state, client, session).await {
            Ok(account) => {
                let _ = app.emit("auth:complete", &account);
            }
            Err(e) => {
                let _ = app.emit("auth:error", e.to_string());
            }
        }
    });

    Ok(challenge)
}

async fn finish_auth(
    app: &AppHandle,
    state: &AccountState,
    client: reqwest::Client,
    session: DeviceCodeSession,
) -> Result<Account, CirrusError> {
    let msa = microsoft::complete_device_code_flow(&client, session).await?;
    let mc = minecraft::authenticate(&client, &msa.access_token).await?;

    store::save_refresh_token(app, &msa.refresh_token)?;

    let account = Account {
        uuid: mc.profile.id.clone(),
        username: mc.profile.name.clone(),
        expires_at: mc.expires_at,
    };

    let mut guard = state.lock().await;
    *guard = Some(InternalSession {
        account: account.clone(),
        mc_token: mc.access_token,
    });

    Ok(account)
}

pub async fn restore_session(
    app: &AppHandle,
    state: &AccountState,
) -> Result<Option<Account>, CirrusError> {
    let client = build_client()?;
    let client_id = resolve_client_id(app)?;

    let Some(refresh) = store::load_refresh_token(app)? else {
        return Ok(None);
    };

    let msa = microsoft::refresh_msa_token(&client, &client_id, &refresh).await?;

    if *msa.refresh_token != *refresh {
        store::save_refresh_token(app, &msa.refresh_token)?;
    }

    let mc = minecraft::authenticate(&client, &msa.access_token).await?;

    let account = Account {
        uuid: mc.profile.id.clone(),
        username: mc.profile.name.clone(),
        expires_at: mc.expires_at,
    };

    let mut guard = state.lock().await;
    *guard = Some(InternalSession {
        account: account.clone(),
        mc_token: mc.access_token,
    });

    Ok(Some(account))
}

pub async fn logout(app: &AppHandle, state: &AccountState) -> Result<(), CirrusError> {
    store::delete_refresh_token(app)?;
    let mut guard = state.lock().await;
    *guard = None;
    Ok(())
}
