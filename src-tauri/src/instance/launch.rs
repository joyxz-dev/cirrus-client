use std::path::PathBuf;
use tauri::AppHandle;
use tokio::process::Command;

use crate::error::CirrusError;
use crate::instance::{cirrus_data_dir, instance_dir, get_instance, save_instance};
use crate::instance::jvm_args::{build_jvm_args, detect_system_ram_mb};

pub async fn launch_instance(
    app: &AppHandle,
    instance_id: &str,
    mc_access_token: &str,
) -> Result<(), CirrusError> {
    let mut instance = get_instance(app, instance_id)?;
    let inst_dir = instance_dir(app, instance_id)?;
    let data_dir = cirrus_data_dir(app)?;

    let java_bin = find_java(&data_dir)?;
    let jar_path = data_dir
        .join("versions")
        .join(&instance.mc_version)
        .join(format!("{}.jar", instance.mc_version));

    if !jar_path.exists() {
        return Err(CirrusError::Launch(format!(
            "Client jar not found for {}. Please download the version first.",
            instance.mc_version
        )));
    }

    let ram = if instance.allocated_ram_mb > 0 {
        instance.allocated_ram_mb
    } else {
        detect_system_ram_mb()
    };

    let mut args = build_jvm_args(ram, &inst_dir);

    // Append user-defined extra JVM args
    args.extend(instance.jvm_args.iter().cloned());

    // Minecraft launch args
    args.push("-jar".into());
    args.push(jar_path.display().to_string());
    args.push("--username".into());
    // Username will be resolved from account state by the caller; we receive it via token
    args.push("Player".into()); // placeholder — real name comes from account state
    args.push("--accessToken".into());
    args.push(mc_access_token.into());
    args.push("--gameDir".into());
    args.push(inst_dir.display().to_string());
    args.push("--assetsDir".into());
    args.push(data_dir.join("assets").display().to_string());
    args.push("--version".into());
    args.push(instance.mc_version.clone());

    if let Some(res) = Some(&instance.resolution) {
        args.push("--width".into());
        args.push(res.width.to_string());
        args.push("--height".into());
        args.push(res.height.to_string());
    }

    let _child = Command::new(&java_bin)
        .args(&args)
        .current_dir(&inst_dir)
        .spawn()
        .map_err(|e| CirrusError::Launch(format!("Failed to spawn JVM: {e}")))?;

    // Update lastPlayedAt
    instance.last_played_at = Some(chrono::Utc::now().timestamp_millis() as u64);
    save_instance(app, &instance)?;

    Ok(())
}

fn find_java(data_dir: &PathBuf) -> Result<PathBuf, CirrusError> {
    // 1. Check managed Java in cirrus data dir
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

    // 2. Fall back to system Java
    let system = PathBuf::from(java_exe());
    if which_java().is_some() {
        return Ok(system);
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
