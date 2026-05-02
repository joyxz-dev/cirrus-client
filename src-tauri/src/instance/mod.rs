pub mod download;
pub mod jvm_args;
pub mod launch;
pub mod sync;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::error::CirrusError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    Fabric,
    Forge,
    Quilt,
    Neoforge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub filename: String,
    pub sha512: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    pub loader_version: Option<String>,
    pub mods: Vec<InstalledMod>,
    pub jvm_args: Vec<String>,
    pub allocated_ram_mb: u32,
    pub sync_options: bool,
    pub resolution: Resolution,
    pub created_at: u64,
    pub last_played_at: Option<u64>,
    pub icon_path: Option<String>,
}

impl Instance {
    pub fn new(
        name: String,
        mc_version: String,
        loader: Loader,
        loader_version: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Instance {
            id: Uuid::new_v4().to_string(),
            name,
            mc_version,
            loader,
            loader_version,
            mods: vec![],
            jvm_args: vec![],
            allocated_ram_mb: 2048,
            sync_options: true,
            resolution: Resolution { width: 1920, height: 1080 },
            created_at: now,
            last_played_at: None,
            icon_path: None,
        }
    }
}

pub fn cirrus_data_dir(app: &AppHandle) -> Result<PathBuf, CirrusError> {
    app.path()
        .app_data_dir()
        .map_err(|e| CirrusError::Instance(e.to_string()))
}

pub fn instance_dir(app: &AppHandle, id: &str) -> Result<PathBuf, CirrusError> {
    validate_id(id)?;
    let base = cirrus_data_dir(app)?;
    let dir = base.join("instances").join(id);
    if !dir.starts_with(&base) {
        return Err(CirrusError::Security("Path traversal detected".into()));
    }
    Ok(dir)
}

fn validate_id(id: &str) -> Result<(), CirrusError> {
    if Uuid::parse_str(id).is_err() {
        return Err(CirrusError::Instance(format!("Invalid instance ID: {id}")));
    }
    Ok(())
}

pub fn list_instances(app: &AppHandle) -> Result<Vec<Instance>, CirrusError> {
    let base = cirrus_data_dir(app)?.join("instances");
    if !base.exists() {
        return Ok(vec![]);
    }

    let mut instances = vec![];
    for entry in std::fs::read_dir(&base)? {
        let entry = entry?;
        let meta_path = entry.path().join("instance.json");
        if meta_path.exists() {
            let data = std::fs::read_to_string(&meta_path)?;
            if let Ok(inst) = serde_json::from_str::<Instance>(&data) {
                instances.push(inst);
            }
        }
    }
    instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(instances)
}

pub fn save_instance(app: &AppHandle, instance: &Instance) -> Result<(), CirrusError> {
    let dir = instance_dir(app, &instance.id)?;
    std::fs::create_dir_all(&dir)?;

    for sub in &["mods", "resourcepacks", "shaderpacks", "saves", "screenshots", "logs", "config"] {
        std::fs::create_dir_all(dir.join(sub))?;
    }

    let meta_path = dir.join("instance.json");
    let json = serde_json::to_string_pretty(instance)?;
    std::fs::write(meta_path, json)?;
    Ok(())
}

pub fn create_instance(
    app: &AppHandle,
    name: String,
    mc_version: String,
    loader: Loader,
    loader_version: Option<String>,
) -> Result<Instance, CirrusError> {
    let inst = Instance::new(name, mc_version, loader, loader_version);
    save_instance(app, &inst)?;
    sync::setup_sync(app, &inst)?;
    Ok(inst)
}

pub fn delete_instance(app: &AppHandle, id: &str) -> Result<(), CirrusError> {
    let dir = instance_dir(app, id)?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub fn get_instance(app: &AppHandle, id: &str) -> Result<Instance, CirrusError> {
    let dir = instance_dir(app, id)?;
    let meta_path = dir.join("instance.json");
    if !meta_path.exists() {
        return Err(CirrusError::Instance(format!("Instance {id} not found")));
    }
    let data = std::fs::read_to_string(meta_path)?;
    serde_json::from_str(&data).map_err(Into::into)
}
