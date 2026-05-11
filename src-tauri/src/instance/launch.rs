use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use tauri::AppHandle;
use tokio::process::Command;
use zeroize::Zeroizing;

use crate::error::CirrusError;
use crate::instance::{cirrus_data_dir, instance_dir, get_instance, save_instance};
use crate::instance::jvm_args::{build_jvm_args, detect_system_ram_mb};

// Only the fields needed to build the launch command
#[derive(Deserialize)]
struct LaunchMeta {
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "assetIndex")]
    asset_index: LaunchAssetRef,
    libraries: Vec<LaunchLib>,
}

#[derive(Deserialize)]
struct LaunchAssetRef {
    id: String,
}

#[derive(Deserialize)]
struct LaunchLib {
    downloads: Option<LaunchLibDownloads>,
    natives: Option<HashMap<String, String>>,
    rules: Option<Vec<LaunchRule>>,
}

#[derive(Deserialize)]
struct LaunchLibDownloads {
    artifact: Option<LaunchArtifact>,
}

#[derive(Deserialize)]
struct LaunchArtifact {
    path: String,
}

#[derive(Deserialize)]
struct LaunchRule {
    action: String,
    os: Option<LaunchOsRule>,
}

#[derive(Deserialize)]
struct LaunchOsRule {
    name: Option<String>,
}

fn current_os() -> &'static str {
    if cfg!(windows) { "windows" }
    else if cfg!(target_os = "macos") { "osx" }
    else { "linux" }
}

fn lib_allowed(lib: &LaunchLib) -> bool {
    let Some(rules) = &lib.rules else { return true };
    let os = current_os();
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

pub async fn launch_instance(
    app: &AppHandle,
    instance_id: &str,
    username: &str,
    uuid: &str,
    mc_token: &Zeroizing<String>,
) -> Result<(), CirrusError> {
    let mut instance = get_instance(app, instance_id)?;
    let inst_dir = instance_dir(app, instance_id)?;
    let data_dir = cirrus_data_dir(app)?;

    let java_bin = find_java(&data_dir)?;

    let versions_dir = data_dir.join("versions").join(&instance.mc_version);
    let jar_path = versions_dir.join(format!("{}.jar", instance.mc_version));
    let meta_path = versions_dir.join(format!("{}.json", instance.mc_version));
    let natives_dir = versions_dir.join("natives");

    if !jar_path.exists() {
        return Err(CirrusError::Launch(format!(
            "Client jar not found for {}. Download it first.",
            instance.mc_version
        )));
    }
    if !meta_path.exists() {
        return Err(CirrusError::Launch(format!(
            "Version metadata not found for {}. Re-download the version.",
            instance.mc_version
        )));
    }

    let meta_text = std::fs::read_to_string(&meta_path)?;
    let meta: LaunchMeta = serde_json::from_str(&meta_text)
        .map_err(|e| CirrusError::Launch(format!("Failed to parse version JSON: {e}")))?;

    let libs_dir = data_dir.join("libraries");

    // Build classpath: platform-filtered libs + client jar
    let mut cp: Vec<String> = meta
        .libraries
        .iter()
        .filter(|l| lib_allowed(l) && l.natives.is_none())
        .filter_map(|l| l.downloads.as_ref()?.artifact.as_ref())
        .map(|a| libs_dir.join(&a.path).display().to_string())
        .collect();
    cp.push(jar_path.display().to_string());

    let cp_sep = if cfg!(windows) { ";" } else { ":" };
    let classpath = cp.join(cp_sep);

    let ram = if instance.allocated_ram_mb > 0 {
        instance.allocated_ram_mb
    } else {
        detect_system_ram_mb()
    };

    let mut args = build_jvm_args(ram);
    args.extend(instance.jvm_args.iter().cloned());

    // Natives path (needed for LWJGL 2; LWJGL 3 ignores it or uses --nativesDirectory)
    std::fs::create_dir_all(&natives_dir)?;
    args.push(format!("-Djava.library.path={}", natives_dir.display()));

    // Classpath and main class
    args.push("-cp".into());
    args.push(classpath);
    args.push(meta.main_class.clone());

    // Game arguments
    args.push("--username".into());
    args.push(username.into());
    args.push("--accessToken".into());
    args.push(mc_token.as_str().into());
    args.push("--uuid".into());
    args.push(uuid.into());
    args.push("--userType".into());
    args.push("mojang".into());
    args.push("--version".into());
    args.push(instance.mc_version.clone());
    args.push("--gameDir".into());
    args.push(inst_dir.display().to_string());
    args.push("--assetsDir".into());
    args.push(data_dir.join("assets").display().to_string());
    args.push("--assetIndex".into());
    args.push(meta.asset_index.id.clone());
    args.push("--nativesDirectory".into());
    args.push(natives_dir.display().to_string());

    if instance.resolution.width > 0 && instance.resolution.height > 0 {
        args.push("--width".into());
        args.push(instance.resolution.width.to_string());
        args.push("--height".into());
        args.push(instance.resolution.height.to_string());
    }

    let _child = Command::new(&java_bin)
        .args(&args)
        .current_dir(&inst_dir)
        .spawn()
        .map_err(|e| CirrusError::Launch(format!("Failed to spawn JVM: {e}")))?;

    instance.last_played_at = Some(chrono::Utc::now().timestamp_millis() as u64);
    save_instance(app, &instance)?;

    Ok(())
}

fn find_java(data_dir: &PathBuf) -> Result<PathBuf, CirrusError> {
    let managed = data_dir.join("java");
    if managed.exists() {
        for entry in std::fs::read_dir(&managed)
            .map_err(|e| CirrusError::Launch(e.to_string()))?
        {
            let entry = entry.map_err(|e| CirrusError::Launch(e.to_string()))?;
            let candidate = entry.path().join("bin").join(java_exe());
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    if which_java().is_some() {
        return Ok(PathBuf::from(java_exe()));
    }

    Err(CirrusError::Launch(
        "Java not found. Please install Java 21 or later.".into(),
    ))
}

fn java_exe() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

fn which_java() -> Option<()> {
    std::process::Command::new("java")
        .arg("-version")
        .output()
        .ok()
        .map(|_| ())
}
