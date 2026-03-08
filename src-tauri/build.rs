fn main() {
    // Brand: read brand.json so config dir and display name can be parameterized.
    let mut publisher = "santiagon610".to_string();
    let mut app_name = "Announcemint".to_string();
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let brand_path = manifest_dir.join("..").join("brand.json");
    if let Ok(data) = std::fs::read_to_string(&brand_path) {
        if let Ok(brand) = json_parse(&data) {
            if let Some(p) = brand.get("publisher") {
                publisher = p.clone();
            }
            if let Some(a) = brand.get("appName") {
                app_name = a.clone();
            }
        }
    }
    println!("cargo:rustc-env=BRAND_PUBLISHER={}", publisher);
    println!("cargo:rustc-env=BRAND_APP_NAME={}", app_name);
    // Embed short Git SHA when building from a git repo (for About screen when not on a release tag).
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                let sha = s.trim();
                if !sha.is_empty() {
                    println!("cargo:rustc-env=GIT_SHA={}", sha);
                }
            }
        }
    }
    tauri_build::build()
}

/// Minimal JSON parse to read "publisher" and "appName" from brand.json without adding a dependency.
fn json_parse(s: &str) -> Result<std::collections::HashMap<String, String>, ()> {
    let mut out = std::collections::HashMap::new();
    for key in ["publisher", "appName"] {
        let needle = format!("\"{}\": \"", key);
        if let Some(start) = s.find(&needle) {
            let value_start = start + needle.len();
            if let Some(end) = s[value_start..].find('"') {
                out.insert(
                    key.to_string(),
                    s[value_start..value_start + end].to_string(),
                );
            }
        }
    }
    if out.is_empty() {
        return Err(());
    }
    Ok(out)
}
