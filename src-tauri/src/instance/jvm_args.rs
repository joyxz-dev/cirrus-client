pub fn build_jvm_args(allocated_ram_mb: u32, instance_path: &std::path::Path) -> Vec<String> {
    let xms = allocated_ram_mb / 2;
    let xmx = allocated_ram_mb;

    vec![
        format!("-Xms{xms}M"),
        format!("-Xmx{xmx}M"),
        "-XX:+UseG1GC".into(),
        "-XX:+ParallelRefProcEnabled".into(),
        "-XX:MaxGCPauseMillis=200".into(),
        "-XX:+UnlockExperimentalVMOptions".into(),
        "-XX:+DisableExplicitGC".into(),
        "-XX:+AlwaysPreTouch".into(),
        "-XX:G1NewSizePercent=30".into(),
        "-XX:G1MaxNewSizePercent=40".into(),
        "-XX:G1HeapRegionSize=8M".into(),
        "-XX:G1ReservePercent=20".into(),
        "-XX:G1HeapWastePercent=5".into(),
        "-XX:G1MixedGCCountTarget=4".into(),
        "-XX:InitiatingHeapOccupancyPercent=15".into(),
        "-XX:G1MixedGCLiveThresholdPercent=90".into(),
        "-XX:G1RSetUpdatingPauseTimePercent=5".into(),
        "-XX:SurvivorRatio=32".into(),
        "-XX:+PerfDisableSharedMem".into(),
        "-XX:MaxTenuringThreshold=1".into(),
        format!("--gameDir={}", instance_path.display()),
    ]
}

pub fn detect_system_ram_mb() -> u32 {
    // Attempt to read from /proc/meminfo (Linux) — on Windows/macOS this will fail and we use default
    #[cfg(target_os = "linux")]
    {
        if let Ok(data) = std::fs::read_to_string("/proc/meminfo") {
            for line in data.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb.parse::<u64>() {
                            let mb = (kb / 1024) as u32;
                            return (mb / 2).clamp(1024, 8192);
                        }
                    }
                }
            }
        }
    }
    2048
}
