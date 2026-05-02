use serde::{Deserialize, Serialize};
use sha1::Digest as Sha1Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

use crate::error::CirrusError;
use crate::instance::cirrus_data_dir;

const VERSION_MANIFEST: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const ASSET_BASE: &str = "https://resources.download.minecraft.net";

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<VersionEntry>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing)]
    url: String,
}

#[derive(Deserialize)]
struct VersionMeta {
    downloads: VersionDownloads,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndexRef,
    libraries: Vec<Library>,
}

#[derive(Deserialize)]
struct VersionDownloads {
    client: DownloadInfo,
}

#[derive(Deserialize)]
struct DownloadInfo {
    url: String,
    sha1: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Deserialize)]
struct AssetIndexRef {
    url: String,
    id: String,
    sha1: String,
}

#[derive(Deserialize)]
struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Deserialize)]
struct Library {
    downloads: Option<LibraryDownloads>,
    #[allow(dead_code)]
    name: String,
}

#[derive(Deserialize)]
struct LibraryDownloads {
    artifact: Option<LibraryArtifact>,
}

#[derive(Deserialize, Clone)]
struct LibraryArtifact {
    url: String,
    sha1: String,
    path: String,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: u64,
    pub total: u64,
}

pub async fn get_version_list(client: &reqwest::Client) -> Result<Vec<VersionEntry>, CirrusError> {
    let manifest: VersionManifest = client
        .get(VERSION_MANIFEST)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(manifest.versions)
}

pub fn is_version_downloaded(app: &AppHandle, mc_version: &str) -> Result<bool, CirrusError> {
    let data_dir = cirrus_data_dir(app)?;
    let jar = data_dir
        .join("versions")
        .join(mc_version)
        .join(format!("{mc_version}.jar"));
    Ok(jar.exists())
}

pub async fn download_version(
    app: &AppHandle,
    client: &reqwest::Client,
    mc_version: &str,
) -> Result<PathBuf, CirrusError> {
    let data_dir = cirrus_data_dir(app)?;
    let versions_dir = data_dir.join("versions").join(mc_version);
    std::fs::create_dir_all(&versions_dir)?;

    let manifest: VersionManifest = client
        .get(VERSION_MANIFEST)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == mc_version)
        .ok_or_else(|| CirrusError::Instance(format!("Version {mc_version} not found")))?;

    let meta: VersionMeta = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    emit_progress(app, "client", 0, 1);

    // Download client jar
    let jar_path = versions_dir.join(format!("{mc_version}.jar"));
    download_and_verify(
        client,
        &meta.downloads.client.url,
        &jar_path,
        &meta.downloads.client.sha1,
        HashAlgo::Sha1,
    )
    .await?;
    emit_progress(app, "client", 1, 1);

    // Download asset index
    let assets_dir = data_dir.join("assets");
    let index_dir = assets_dir.join("indexes");
    std::fs::create_dir_all(&index_dir)?;
    let index_path = index_dir.join(format!("{}.json", meta.asset_index.id));
    download_and_verify(
        client,
        &meta.asset_index.url,
        &index_path,
        &meta.asset_index.sha1,
        HashAlgo::Sha1,
    )
    .await?;

    // Download assets
    let objects_dir = assets_dir.join("objects");
    let index_data = std::fs::read_to_string(&index_path)?;
    let asset_index: AssetIndex = serde_json::from_str(&index_data)?;
    let total_assets = asset_index.objects.len() as u64;
    let mut done = 0u64;

    for (_, obj) in &asset_index.objects {
        let prefix = &obj.hash[..2];
        let obj_dir = objects_dir.join(prefix);
        std::fs::create_dir_all(&obj_dir)?;
        let obj_path = obj_dir.join(&obj.hash);
        let url = format!("{ASSET_BASE}/{prefix}/{}", obj.hash);
        download_and_verify(client, &url, &obj_path, &obj.hash, HashAlgo::Sha1).await?;
        done += 1;
        if done % 50 == 0 || done == total_assets {
            emit_progress(app, "assets", done, total_assets);
        }
    }

    // Download libraries
    let libs_dir = data_dir.join("libraries");
    let libs: Vec<_> = meta
        .libraries
        .iter()
        .filter_map(|l| l.downloads.as_ref()?.artifact.as_ref().map(|a| a.clone()))
        .collect();
    let total_libs = libs.len() as u64;
    let mut lib_done = 0u64;

    for artifact in &libs {
        let lib_path = libs_dir.join(&artifact.path);
        if let Some(parent) = lib_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        download_and_verify(client, &artifact.url, &lib_path, &artifact.sha1, HashAlgo::Sha1)
            .await?;
        lib_done += 1;
        if lib_done % 10 == 0 || lib_done == total_libs {
            emit_progress(app, "libraries", lib_done, total_libs);
        }
    }

    Ok(jar_path)
}

fn emit_progress(app: &AppHandle, phase: &str, current: u64, total: u64) {
    let _ = app.emit(
        "download:progress",
        DownloadProgress {
            phase: phase.to_string(),
            current,
            total,
        },
    );
}

#[allow(dead_code)]
pub(crate) enum HashAlgo {
    Sha1,
    Sha512,
}

pub async fn download_and_verify(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_hash: &str,
    algo: HashAlgo,
) -> Result<(), CirrusError> {
    if dest.exists() {
        let data = std::fs::read(dest)?;
        if verify_hash(&data, expected_hash, &algo) {
            return Ok(());
        }
        std::fs::remove_file(dest)?;
    }

    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    if !verify_hash(&bytes, expected_hash, &algo) {
        return Err(CirrusError::Security(format!("Hash mismatch for {url}")));
    }

    std::fs::write(dest, &bytes)?;
    Ok(())
}

fn verify_hash(data: &[u8], expected: &str, algo: &HashAlgo) -> bool {
    let computed = match algo {
        HashAlgo::Sha1 => {
            let mut h = sha1::Sha1::new();
            h.update(data);
            hex::encode(h.finalize())
        }
        HashAlgo::Sha512 => {
            let mut h = sha2::Sha512::new();
            h.update(data);
            hex::encode(h.finalize())
        }
    };
    computed == expected
}
