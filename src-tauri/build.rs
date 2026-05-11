fn main() {
    // Inject AZURE_CLIENT_ID from .env so option_env! picks it up at compile time.
    if let Ok(contents) = std::fs::read_to_string("../.env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                if key == "AZURE_CLIENT_ID" && !val.is_empty() {
                    println!("cargo:rustc-env=AZURE_CLIENT_ID={val}");
                }
            }
        }
    }
    println!("cargo:rerun-if-changed=../.env");
    tauri_build::build()
}
