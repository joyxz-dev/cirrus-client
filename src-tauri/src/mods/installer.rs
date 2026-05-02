use std::path::Path;
use tauri::AppHandle;

use crate::error::CirrusError;
use crate::instance::{get_instance, instance_dir, save_instance, InstalledMod};
use crate::mods::{build_client, get_mod_versions, ModVersionFile};

pub async fn install_mod(
    app: &AppHandle,
    instance_id: &str,
    project_id: &str,
    version_id: Option<&str>,
) -> Result<InstalledMod, CirrusError> {
    let instance = get_instance(app, instance_id)?;
    let client = build_client()?;

    let loader_str = format!("{:?}", instance.loader).to_lowercase();
    let versions = get_mod_versions(
        &client,
        project_id,
        Some(&loader_str),
        Some(&instance.mc_version),
    )
    .await?;

    let version = if let Some(vid) = version_id {
        versions
            .iter()
            .find(|v| v.id == vid)
            .ok_or_else(|| CirrusError::Instance(format!("Version {vid} not found")))?
    } else {
        versions
            .first()
            .ok_or_else(|| CirrusError::Instance("No compatible version found".into()))?
    };

    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| CirrusError::Instance("No downloadable file in version".into()))?;

    let inst_dir = instance_dir(app, instance_id)?;
    let mods_dir = inst_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)?;

    let dest = mods_dir.join(&file.filename);
    download_mod_file(&client, file, &dest).await?;

    let installed = InstalledMod {
        id: project_id.to_string(),
        name: version.name.clone(),
        version: version.version_number.clone(),
        filename: file.filename.clone(),
        sha512: file.hashes.sha512.clone(),
    };

    let mut inst = get_instance(app, instance_id)?;
    inst.mods.retain(|m| m.id != project_id);
    inst.mods.push(installed.clone());
    save_instance(app, &inst)?;

    Ok(installed)
}

async fn download_mod_file(
    client: &reqwest::Client,
    file: &ModVersionFile,
    dest: &Path,
) -> Result<(), CirrusError> {
    use sha2::Digest;

    let bytes = client
        .get(&file.url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let mut hasher = sha2::Sha512::new();
    hasher.update(&bytes);
    let computed = hex::encode(hasher.finalize());

    if computed != file.hashes.sha512 {
        return Err(CirrusError::Security(format!(
            "SHA512 mismatch for {}",
            file.filename
        )));
    }

    std::fs::write(dest, &bytes)?;
    Ok(())
}

pub fn remove_mod(
    app: &AppHandle,
    instance_id: &str,
    mod_id: &str,
) -> Result<(), CirrusError> {
    let mut instance = get_instance(app, instance_id)?;
    let inst_dir = instance_dir(app, instance_id)?;

    if let Some(m) = instance.mods.iter().find(|m| m.id == mod_id) {
        let jar_path = inst_dir.join("mods").join(&m.filename);
        if jar_path.exists() {
            std::fs::remove_file(jar_path)?;
        }
    }

    instance.mods.retain(|m| m.id != mod_id);
    save_instance(app, &instance)?;
    Ok(())
}
