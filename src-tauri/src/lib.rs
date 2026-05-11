mod auth;
mod error;
mod instance;
mod mods;
mod store;

use auth::{Account, AccountState};
use error::CirrusError;
use instance::{Loader, Instance, InstalledMod};
use mods::{ModSearchQuery, ModSearchResult, ModVersion};
use tauri::{AppHandle, Emitter, State};

fn http_client() -> Result<reqwest::Client, CirrusError> {
    reqwest::Client::builder()
        .user_agent("Cirrus/0.1.0 (cirrusclient.gg)")
        .build()
        .map_err(Into::into)
}

// ── Auth commands ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_account(state: State<'_, AccountState>) -> Result<Option<Account>, CirrusError> {
    Ok(state.lock().await.as_ref().map(|s| s.account.clone()))
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
fn get_client_id(app: AppHandle) -> Result<Option<String>, CirrusError> {
    store::load_client_id(&app)
}

#[tauri::command]
fn set_client_id(app: AppHandle, client_id: String) -> Result<(), CirrusError> {
    let trimmed = client_id.trim().to_string();
    if trimmed.is_empty() {
        return Err(CirrusError::Auth("Client ID cannot be empty".into()));
    }
    store::save_client_id(&app, &trimmed)
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
fn update_instance(
    app: AppHandle,
    id: String,
    name: Option<String>,
    allocated_ram_mb: Option<u32>,
    resolution_width: Option<u32>,
    resolution_height: Option<u32>,
) -> Result<Instance, CirrusError> {
    instance::update_instance(&app, &id, name, allocated_ram_mb, resolution_width, resolution_height)
}

#[tauri::command]
fn delete_instance(app: AppHandle, id: String) -> Result<(), CirrusError> {
    instance::delete_instance(&app, &id)
}

#[tauri::command]
fn get_instance(app: AppHandle, id: String) -> Result<Instance, CirrusError> {
    instance::get_instance(&app, &id)
}

/// Download version files if not already cached, then launch the game.
/// Emits `download:progress` events during download.
#[tauri::command]
async fn launch_instance(
    app: AppHandle,
    state: State<'_, AccountState>,
    id: String,
) -> Result<(), CirrusError> {
    let (username, uuid, mc_token, expires_at) = {
        let guard = state.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| CirrusError::Auth("Not logged in".into()))?;
        (
            session.account.username.clone(),
            session.account.uuid.clone(),
            session.mc_token.clone(),
            session.account.expires_at,
        )
    };

    if chrono::Utc::now().timestamp() as u64 >= expires_at {
        return Err(CirrusError::Auth(
            "Session expired. Please sign out and sign in again.".into(),
        ));
    }

    let inst = instance::get_instance(&app, &id)?;

    if !instance::download::is_version_downloaded(&app, &inst.mc_version)? {
        let client = http_client()?;
        instance::download::download_version(&app, &client, &inst.mc_version).await?;
    }

    instance::launch::launch_instance(&app, &id, &username, &uuid, &mc_token).await
}

#[tauri::command]
async fn download_version(app: AppHandle, mc_version: String) -> Result<(), CirrusError> {
    let client = http_client()?;
    instance::download::download_version(&app, &client, &mc_version).await?;
    Ok(())
}

#[tauri::command]
async fn get_version_list() -> Result<Vec<instance::download::VersionEntry>, CirrusError> {
    let client = http_client()?;
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

// ── Defaults commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn get_defaults(app: AppHandle) -> Result<(u32, u32, u32), CirrusError> {
    Ok(store::load_defaults(&app))
}

#[tauri::command]
fn set_defaults(app: AppHandle, ram_mb: u32, width: u32, height: u32) -> Result<(), CirrusError> {
    store::save_defaults(&app, ram_mb.clamp(512, 65536), width, height)
}

#[tauri::command]
fn open_instance_folder(app: AppHandle, id: String) -> Result<(), CirrusError> {
    let dir = instance::instance_dir(&app, &id)?;
    if !dir.exists() {
        return Err(CirrusError::Instance("Instance folder not found".into()));
    }
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().ok();
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&dir).spawn().ok();
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().ok();
    Ok(())
}

const ALLOWED_SUBFOLDERS: &[&str] = &["mods", "resourcepacks", "shaderpacks", "saves", "screenshots"];

#[tauri::command]
fn list_instance_folder(app: AppHandle, id: String, subfolder: String) -> Result<Vec<String>, CirrusError> {
    if !ALLOWED_SUBFOLDERS.contains(&subfolder.as_str()) {
        return Err(CirrusError::Security("Subfolder not allowed".into()));
    }
    let dir = instance::instance_dir(&app, &id)?.join(&subfolder);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names = vec![];
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
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
fn remove_mod(app: AppHandle, instance_id: String, mod_id: String) -> Result<(), CirrusError> {
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
            let app_handle = app.handle().clone();
            let state = account_state.clone();
            tauri::async_runtime::spawn(async move {
                match auth::restore_session(&app_handle, &state).await {
                    Ok(Some(account)) => {
                        let _ = app_handle.emit("auth:complete", &account);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        log::warn!("Session restore failed: {e}");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_account,
            get_client_id,
            set_client_id,
            start_auth,
            logout,
            list_instances,
            create_instance,
            update_instance,
            delete_instance,
            get_instance,
            launch_instance,
            download_version,
            get_version_list,
            set_sync_options,
            get_defaults,
            set_defaults,
            open_instance_folder,
            list_instance_folder,
            search_mods,
            get_mod_versions,
            install_mod,
            remove_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cirrus");
}
