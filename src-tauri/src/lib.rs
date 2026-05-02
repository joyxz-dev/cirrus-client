mod auth;
mod error;
mod instance;
mod mods;
mod store;

use auth::{Account, AccountState};
use error::CirrusError;
use instance::{Loader, Instance, InstalledMod};
use mods::{ModSearchQuery, ModSearchResult, ModVersion};
use tauri::{AppHandle, State};

// ── Auth commands ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_account(state: State<'_, AccountState>) -> Result<Option<Account>, CirrusError> {
    Ok(state.lock().await.clone())
}

#[tauri::command]
async fn start_auth(
    app: AppHandle,
    state: State<'_, AccountState>,
) -> Result<auth::microsoft::DeviceCodeChallenge, CirrusError> {
    let state = state.inner().clone();
    auth::start_auth(&app, &state).await
}

#[tauri::command]
async fn logout(
    app: AppHandle,
    state: State<'_, AccountState>,
) -> Result<(), CirrusError> {
    auth::logout(&app, state.inner()).await
}

// ── Instance commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn list_instances(app: AppHandle) -> Result<Vec<Instance>, CirrusError> {
    instance::list_instances(&app)
}

#[tauri::command]
fn create_instance(
    app: AppHandle,
    name: String,
    mc_version: String,
    loader: Loader,
    loader_version: Option<String>,
) -> Result<Instance, CirrusError> {
    instance::create_instance(&app, name, mc_version, loader, loader_version)
}

#[tauri::command]
fn delete_instance(app: AppHandle, id: String) -> Result<(), CirrusError> {
    instance::delete_instance(&app, &id)
}

#[tauri::command]
fn get_instance(app: AppHandle, id: String) -> Result<Instance, CirrusError> {
    instance::get_instance(&app, &id)
}

#[tauri::command]
async fn launch_instance(
    app: AppHandle,
    state: State<'_, AccountState>,
    id: String,
) -> Result<(), CirrusError> {
    let guard = state.lock().await;
    let _account = guard
        .as_ref()
        .ok_or_else(|| CirrusError::Auth("Not logged in".into()))?;
    drop(guard);

    // NOTE: access token is not passed to the frontend — we retrieve it here from in-memory state
    // For Phase 1, we pass a placeholder; Phase 2 will wire in the real MC access token
    instance::launch::launch_instance(&app, &id, "0").await
}

#[tauri::command]
async fn download_version(
    app: AppHandle,
    mc_version: String,
) -> Result<(), CirrusError> {
    let client = reqwest::Client::builder()
        .user_agent("Cirrus/0.1.0 (cirrusclient.gg)")
        .build()
        .map_err(CirrusError::from)?;
    instance::download::download_version(&app, &client, &mc_version).await?;
    Ok(())
}

#[tauri::command]
async fn get_version_list() -> Result<Vec<String>, CirrusError> {
    let client = reqwest::Client::builder()
        .user_agent("Cirrus/0.1.0 (cirrusclient.gg)")
        .build()
        .map_err(CirrusError::from)?;
    instance::download::get_version_list(&client).await
}

#[tauri::command]
fn set_sync_options(
    app: AppHandle,
    instance_id: String,
    enabled: bool,
) -> Result<(), CirrusError> {
    let mut inst = instance::get_instance(&app, &instance_id)?;
    if enabled && !inst.sync_options {
        instance::sync::enable_sync(&app, &instance_id)?;
    } else if !enabled && inst.sync_options {
        instance::sync::disable_sync(&app, &instance_id)?;
    }
    inst.sync_options = enabled;
    instance::save_instance(&app, &inst)
}

// ── Mods commands ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn search_mods(query: ModSearchQuery) -> Result<ModSearchResult, CirrusError> {
    let client = mods::build_client()?;
    mods::search_mods(&client, &query).await
}

#[tauri::command]
async fn get_mod_versions(
    project_id: String,
    loader: Option<String>,
    mc_version: Option<String>,
) -> Result<Vec<ModVersion>, CirrusError> {
    let client = mods::build_client()?;
    mods::get_mod_versions(&client, &project_id, loader.as_deref(), mc_version.as_deref()).await
}

#[tauri::command]
async fn install_mod(
    app: AppHandle,
    instance_id: String,
    project_id: String,
    version_id: Option<String>,
) -> Result<InstalledMod, CirrusError> {
    mods::installer::install_mod(&app, &instance_id, &project_id, version_id.as_deref()).await
}

#[tauri::command]
fn remove_mod(
    app: AppHandle,
    instance_id: String,
    mod_id: String,
) -> Result<(), CirrusError> {
    mods::installer::remove_mod(&app, &instance_id, &mod_id)
}

// ── App entry point ───────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let account_state = auth::new_account_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .manage(account_state.clone())
        .setup(move |app| {
            // Attempt to restore session from stored refresh token on startup
            let app_handle = app.handle().clone();
            let state = account_state.clone();
            tokio::spawn(async move {
                if let Err(e) = auth::restore_session(&app_handle, &state).await {
                    log::warn!("Session restore failed: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_account,
            start_auth,
            logout,
            list_instances,
            create_instance,
            delete_instance,
            get_instance,
            launch_instance,
            download_version,
            get_version_list,
            set_sync_options,
            search_mods,
            get_mod_versions,
            install_mod,
            remove_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cirrus");
}
