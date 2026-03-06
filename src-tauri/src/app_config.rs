//! Shared app config: load/save and credential options for both GUI and CLI.

use crate::polly::AwsCredentialOptions;
use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_FILENAME: &str = "config.json";
const CONFIG_DIR_PUBLISHER: &str = env!("BRAND_PUBLISHER");
const CONFIG_DIR_APP: &str = env!("BRAND_APP_NAME");

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub voice_id: Option<String>,
    pub engine: Option<String>,
    pub language_code: Option<String>,
    pub preset_name: Option<String>,
    pub output_dir: Option<String>,
    pub prompt_lines: Option<String>,
    pub remember_prompts: Option<bool>,
    pub prompt_file_name_format: Option<String>,
    pub aws_proxy_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_proxy_url: Option<String>,
    pub aws_proxy_protocol: Option<String>,
    pub aws_proxy_host: Option<String>,
    pub aws_proxy_port: Option<String>,
    pub aws_proxy_username: Option<String>,
    pub aws_proxy_password: Option<String>,
    pub aws_profile: Option<String>,
    pub aws_config_dir: Option<String>,
    pub aws_region_manual: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_use_manual: Option<bool>,
}

pub fn app_config_dir() -> Result<std::path::PathBuf, String> {
    dirs::config_dir()
        .ok_or_else(|| "No config directory".to_string())
        .map(|p| p.join(CONFIG_DIR_PUBLISHER).join(CONFIG_DIR_APP))
}

/// Load config from default path or from an override path. Returns default config if file is missing.
pub fn load_app_config(config_file_override: Option<&Path>) -> Result<AppConfig, String> {
    let path = match config_file_override {
        Some(p) => p.to_path_buf(),
        None => {
            let dir = app_config_dir()?;
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            dir.join(CONFIG_FILENAME)
        }
    };
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let s = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

pub fn config_path() -> Result<std::path::PathBuf, String> {
    let dir = app_config_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(CONFIG_FILENAME))
}

pub fn build_proxy_url(c: &AppConfig) -> Option<String> {
    if !c.aws_proxy_enabled.unwrap_or(false) {
        return None;
    }
    let host = c.aws_proxy_host.as_deref().filter(|s| !s.is_empty())?;
    let port = c
        .aws_proxy_port
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| match c.aws_proxy_protocol.as_deref().unwrap_or("http") {
            "https" => "443",
            "socks" | "socks5" => "1080",
            _ => "8080",
        });
    let protocol = match c.aws_proxy_protocol.as_deref().unwrap_or("http") {
        "https" => "https",
        "socks" | "socks5" => "socks5",
        _ => "http",
    };
    let user = c.aws_proxy_username.as_deref().filter(|s| !s.is_empty());
    let pass = c.aws_proxy_password.as_ref();
    let auth = if user.is_some() || pass.as_ref().map_or(false, |s| !s.is_empty()) {
        let u = urlencoding::encode(user.unwrap_or(""));
        let p = urlencoding::encode(pass.as_deref().map_or("", |v| v));
        format!("{}:{}@", u, p)
    } else {
        String::new()
    };
    Some(format!("{}://{}{}:{}", protocol, auth, host, port))
}

pub fn config_to_credential_options(c: &AppConfig) -> AwsCredentialOptions {
    let use_manual = c.aws_use_manual.unwrap_or_else(|| {
        c.aws_access_key_id
            .as_ref()
            .map_or(false, |s| !s.is_empty())
            && c.aws_secret_access_key
                .as_ref()
                .map_or(false, |s| !s.is_empty())
    });
    let proxy_url = build_proxy_url(c).or_else(|| c.aws_proxy_url.clone());
    AwsCredentialOptions {
        profile: c.aws_profile.clone(),
        config_dir: c.aws_config_dir.clone(),
        region: c.aws_region_manual.clone(),
        proxy_enabled: c.aws_proxy_enabled.unwrap_or(false),
        proxy_url,
        access_key_id: if use_manual {
            c.aws_access_key_id.clone()
        } else {
            None
        },
        secret_access_key: if use_manual {
            c.aws_secret_access_key.clone()
        } else {
            None
        },
    }
}
