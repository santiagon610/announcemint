//! Helpers for AWS config/credentials file paths and listing profiles.

use std::path::{Path, PathBuf};

/// Default directory for AWS config and credentials files.
/// On Unix: ~/.aws, on Windows: %USERPROFILE%\.aws
pub fn default_aws_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".aws"))
}

/// Path to the credentials file in the given config directory (or default).
pub fn credentials_file_path(config_dir: Option<&Path>) -> Option<PathBuf> {
    let dir = config_dir
        .map(PathBuf::from)
        .or_else(default_aws_config_dir)?;
    Some(dir.join("credentials"))
}

/// Path to the config file in the given config directory (or default).
pub fn config_file_path(config_dir: Option<&Path>) -> Option<PathBuf> {
    let dir = config_dir
        .map(PathBuf::from)
        .or_else(default_aws_config_dir)?;
    Some(dir.join("config"))
}

/// List profile names from an AWS credentials file (INI-style [section] names).
/// Credentials file uses [default], [profilename] (no "profile " prefix).
/// Returns empty vec if the file is missing or unreadable.
pub fn list_profiles_from_credentials_file(credentials_path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(credentials_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut profiles = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim();
            if !name.is_empty() && !name.starts_with("profile ") {
                profiles.push(name.to_string());
            }
        }
    }
    profiles
}

/// List profile names from an AWS config file (INI-style).
/// Config file uses [default] and [profile profilename] for named profiles.
/// Returns empty vec if the file is missing or unreadable.
pub fn list_profiles_from_config_file(config_path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut profiles = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim();
            if name.is_empty() {
                continue;
            }
            if name == "default" {
                profiles.push("default".to_string());
            } else if let Some(profile_name) = name.strip_prefix("profile ") {
                let profile_name = profile_name.trim();
                if !profile_name.is_empty() {
                    profiles.push(profile_name.to_string());
                }
            }
        }
    }
    profiles
}

/// List profile names from the default or given config directory.
/// Reads both the credentials file and the config file, merges and deduplicates
/// (config file uses [profile name], credentials uses [name]; both are needed).
pub fn list_aws_profiles(config_dir: Option<&Path>) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    if let Some(cred_path) = credentials_file_path(config_dir) {
        if cred_path.exists() {
            for p in list_profiles_from_credentials_file(&cred_path) {
                if seen.insert(p.clone()) {
                    out.push(p);
                }
            }
        }
    }
    if let Some(cfg_path) = config_file_path(config_dir) {
        if cfg_path.exists() {
            for p in list_profiles_from_config_file(&cfg_path) {
                if seen.insert(p.clone()) {
                    out.push(p);
                }
            }
        }
    }
    out
}

/// Get the region for a profile from the AWS config file.
/// For "default" uses [default]; for other profiles uses [profile name].
/// Returns None if the file is missing, unreadable, or the profile/section has no region set.
pub fn get_profile_region(config_dir: Option<&Path>, profile_name: &str) -> Option<String> {
    let path = config_file_path(config_dir)?;
    let content = std::fs::read_to_string(&path).ok()?;
    let mut in_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim();
            in_section = if profile_name == "default" {
                name == "default"
            } else {
                name == format!("profile {}", profile_name)
            };
            continue;
        }
        if in_section && line.starts_with("region") {
            let rest = line.strip_prefix("region")?.trim();
            let rest = rest.trim_start_matches('=').trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        list_profiles_from_config_file, list_profiles_from_credentials_file, list_aws_profiles,
    };
    use std::io::Write;

    #[test]
    fn test_list_profiles_credentials() {
        let dir = std::env::temp_dir();
        let path = dir.join("test-credentials");
        let content = r#"
[default]
aws_access_key_id = AKIA...

[profile dev]
aws_access_key_id = AKIA...

[staging]
aws_access_key_id = AKIA...
"#;
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        let profiles = list_profiles_from_credentials_file(&path);
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"staging".to_string()));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_list_profiles_config() {
        let dir = std::env::temp_dir();
        let path = dir.join("test-config");
        let content = r#"
[default]
region = us-east-1

[profile dev]
region = us-west-2

[profile btvfc-root]
region = us-east-1
sso_start_url = https://example.awsapps.com/start
"#;
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        let profiles = list_profiles_from_config_file(&path);
        assert_eq!(profiles.len(), 3);
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"dev".to_string()));
        assert!(profiles.contains(&"btvfc-root".to_string()));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_list_aws_profiles_merges_both_files() {
        let dir = std::env::temp_dir();
        let aws_dir = dir.join("aws-merge-test");
        std::fs::create_dir_all(&aws_dir).unwrap();
        let cred_path = aws_dir.join("credentials");
        let config_path = aws_dir.join("config");
        std::fs::write(
            &cred_path,
            "[default]\naws_access_key_id = x\n[only-in-cred]\naws_access_key_id = y\n",
        )
        .unwrap();
        std::fs::write(
            &config_path,
            "[default]\nregion = us-east-1\n[profile only-in-config]\nregion = us-east-1\n",
        )
        .unwrap();
        let profiles = list_aws_profiles(Some(aws_dir.as_path()));
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"only-in-cred".to_string()));
        assert!(profiles.contains(&"only-in-config".to_string()));
        std::fs::remove_file(cred_path).ok();
        std::fs::remove_file(config_path).ok();
        std::fs::remove_dir(aws_dir).ok();
    }
}
