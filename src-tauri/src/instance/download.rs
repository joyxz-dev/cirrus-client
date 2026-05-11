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
    natives: Option<HashMap<String, String>>,
    rules: Option<Vec<LibraryRule>>,
}

#[derive(Deserialize)]
struct LibraryDownloads {
    artifact: Option<LibraryArtifact>,
    classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Deserialize, Clone)]
struct LibraryArtifact {
    url: String,
    sha1: String,
    path: String,
}

#[derive(Deserialize)]
struct LibraryRule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Deserialize)]
struct OsRule {
    name: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: u64,
    pub total: u64,
}

fn platform_os() -> &'static str {
    if cfg!(windows) { "windows" }
    else if cfg!(target_os = "macos") { "osx" }
    else { "linux" }
}

fn lib_allowed(rules: &Option<Vec<LibraryRule>>) -> bool {
    let Some(rules) = rules else { return true };
    let os = platform_os();
    let mut allowed = false;
    for rule in rules {
        let matches = rule.os.as_ref()
            .and_then(|o| o.name.as_deref())
            .map_or(true, |name| name == os);
        if matches {
            allowed = rule.action == "allow";
        }
    }
    allowed
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
    let base = data_dir.join("versions").join(mc_version);
    Ok(base.join(format!("{mc_version}.jar")).exists()
        && base.join(format!("{mc_version}.json")).exists())
}

pub async fn download_version(
    app: &AppHandle,
    client: &reqwest::Client,
    mc_version: &str,
) -> Result<PathBuf, CirrusError> {
    let data_dir = cirrus_data_dir(app)?;
    let versions_dir = data_dir.join("versions").join(mc_version);
    std::fs::create_dir_all(&versions_dir)?;

    // Fetch version manifest
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

    // Fetch and save version JSON (needed at launch time for mainClass etc.)
    let meta_json_path = versions_dir.join(format!("{mc_version}.json"));
    let meta_text = client
        .get(&entry.url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let meta: VersionMeta = serde_json::from_str(&meta_text)?;
    std::fs::write(&meta_json_path, &meta_text)?;

    emit_progress(app, "client", 0, 1);

    // Client jar
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

    // Asset index
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

    // Assets
    let objects_dir = assets_dir.join("objects");
    let index_data = std::fs::read_to_string(&index_path)?;
    let asset_index: AssetIndex = serde_json::from_str(&index_data)?;
    let total_assets = asset_index.objects.len() as u64;
    let mut done = 0u64;

    for obj in asset_index.objects.values() {
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

    // Libraries
    let libs_dir = data_dir.join("libraries");
    let lib_artifacts: Vec<_> = meta
        .libraries
        .iter()
        .filter(|l| lib_allowed(&l.rules))
        .filter_map(|l| l.downloads.as_ref()?.artifact.as_ref().map(|a| a.clone()))
        .collect();
    let total_libs = lib_artifacts.len() as u64;
    let mut lib_done = 0u64;

    for artifact in &lib_artifacts {
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

    // Native libraries
    let natives_dir = versions_dir.join("natives");
    std::fs::create_dir_all(&natives_dir)?;
    download_natives(client, &meta.libraries, &libs_dir, &natives_dir).await?;

    Ok(jar_path)
}

async fn download_natives(
    client: &reqwest::Client,
    libs: &[Library],
    libs_dir: &Path,
    natives_dir: &Path,
) -> Result<(), CirrusError> {
    let os = platform_os();

    for lib in libs {
        let Some(natives_map) = &lib.natives else { continue };
        let Some(classifier_key) = natives_map.get(os) else { continue };
        let Some(downloads) = &lib.downloads else { continue };
        let Some(classifiers) = &downloads.classifiers else { continue };
        let Some(artifact) = classifiers.get(classifier_key.as_str()) else { continue };

        let jar_path = libs_dir.join(&artifact.path);
        if let Some(parent) = jar_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        download_and_verify(client, &artifact.url, &jar_path, &artifact.sha1, HashAlgo::Sha1)
            .await?;
        extract_natives(&jar_path, natives_dir)?;
    }
    Ok(())
}

fn extract_natives(jar_path: &Path, dest_dir: &Path) -> Result<(), CirrusError> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| CirrusError::Instance(format!("Failed to open native JAR: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CirrusError::Instance(format!("ZIP read error: {e}")))?;

        let name = entry.name().to_string();
        if entry.is_dir() || name.starts_with("META-INF/") {
            continue;
        }

        let out_path = dest_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
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
