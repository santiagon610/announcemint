// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_config;
mod aws_config_file;
mod cli;
mod polly;
mod preset;
mod progress;

use app_config::{
    build_proxy_url, config_path, config_to_credential_options, load_app_config, AppConfig,
};
use polly::{
    build_client_with_options, check_session, describe_voices, format_prompt_filename,
    load_sdk_config_with_options, resolve_region, synthesize_line, test_synthesize_speech,
    VoiceInfo,
};
use preset::{apply_preset, OutputFormat, OutputPreset};
use progress::ProgressReporter;
use serde::Serialize;
use std::path::Path;
use tauri::Emitter;
use tokio::sync::mpsc;

fn apply_proxy_env_from_config(c: &AppConfig) {
    if let Some(url) = build_proxy_url(c) {
        std::env::set_var("HTTP_PROXY", &url);
        std::env::set_var("HTTPS_PROXY", &url);
    }
}

/// Fetch public IP for diagnostics (uses HTTP_PROXY/HTTPS_PROXY if set).
async fn fetch_public_ip() -> Option<String> {
    let client = reqwest::Client::builder().build().ok()?;
    let body = client
        .get("https://checkip.amazonaws.com")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let ip = body.trim().to_string();
    if ip.is_empty() || !ip.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    Some(ip)
}

/// Turn AWS SDK / credential errors into clearer messages for the user.
fn normalize_aws_error(e: impl AsRef<str>) -> String {
    let s = e.as_ref();
    let lower = s.to_lowercase();
    if lower.contains("dispatch failure") {
        return "Unable to get credentials for this profile. If you use SSO, run `aws sso login` for this profile and try again. Otherwise check that the profile name and config path are correct.".to_string();
    }
    if lower.contains("unable to load credentials")
        || lower.contains("could not load credentials")
        || lower.contains("failed to load credentials")
    {
        return format!(
            "Credentials could not be loaded. {} If using SSO, run `aws sso login` for this profile.",
            s
        );
    }
    if lower.contains("expired") || (lower.contains("sso") && lower.contains("token")) {
        return format!(
            "Session or token has expired. {} Run `aws sso login` for this profile and try again.",
            s
        );
    }
    if lower.contains("sso") && (lower.contains("login") || lower.contains("refresh")) {
        return format!("SSO session may be expired or invalid. Run `aws sso login` for this profile. Details: {}", s);
    }
    if lower.contains("profile") && lower.contains("not found") {
        return format!("Profile not found or not configured. {}", s);
    }
    if s.is_empty() || lower == "error" {
        return "Authentication failed. Check your profile and credentials.".to_string();
    }
    s.to_string()
}

#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    load_app_config(None)
}

#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    let path = config_path()?;
    let s = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, s).await.map_err(|e| e.to_string())
}

/// Deletes the app config file. Used by the Danger Zone "Reset settings" flow; caller should relaunch after.
#[tauri::command]
fn delete_config_file() -> Result<(), String> {
    let path = config_path()?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn list_presets() -> Vec<OutputPreset> {
    OutputPreset::builtins()
}

#[tauri::command]
fn ping() -> String {
    "pong".into()
}

/// Returns the system color scheme on Linux via the XDG Settings portal (org.freedesktop.appearance.color-scheme).
/// This fixes dark mode when the GTK theme name does not indicate dark (e.g. "Adwaita" with prefer-dark).
/// On non-Linux or if the portal is unavailable, returns None so the frontend can use the window theme API.
#[tauri::command]
async fn get_system_theme() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        use ashpd::desktop::settings::{ColorScheme, Settings};
        let settings = Settings::new().await.ok()?;
        let scheme = settings.color_scheme().await.ok()?;
        Some(match scheme {
            ColorScheme::PreferDark => "dark".to_string(),
            ColorScheme::PreferLight => "light".to_string(),
            ColorScheme::NoPreference => "light".to_string(),
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[tauri::command]
fn get_default_aws_config_dir() -> Option<String> {
    aws_config_file::default_aws_config_dir()
        .filter(|p| p.exists())
        .and_then(|p| p.into_os_string().into_string().ok())
}

#[tauri::command]
fn list_aws_profiles(config_dir: Option<String>) -> Vec<String> {
    let dir = config_dir.as_deref().map(Path::new);
    aws_config_file::list_aws_profiles(dir)
}

#[tauri::command]
fn get_aws_profile_env() -> Option<String> {
    std::env::var("AWS_PROFILE").ok().filter(|s| !s.is_empty())
}

#[tauri::command]
fn get_profile_region(config_dir: Option<String>, profile_name: String) -> Option<String> {
    let dir = config_dir.as_deref().map(Path::new);
    aws_config_file::get_profile_region(dir, &profile_name)
}

/// AWS region codes where Amazon Polly has an endpoint (see https://docs.aws.amazon.com/general/latest/gr/pol.html).
fn aws_regions_list() -> Vec<&'static str> {
    vec![
        "us-east-1",
        "us-east-2",
        "us-west-1",
        "us-west-2",
        "af-south-1",
        "ap-east-1",
        "ap-south-1",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-southeast-5",
        "ap-northeast-1",
        "ap-northeast-2",
        "ap-northeast-3",
        "ca-central-1",
        "eu-central-1",
        "eu-central-2",
        "eu-west-1",
        "eu-west-2",
        "eu-west-3",
        "eu-north-1",
        "eu-south-2",
        "me-south-1",
        "sa-east-1",
        "us-gov-west-1",
    ]
}

#[tauri::command]
fn list_aws_regions() -> Vec<String> {
    aws_regions_list().into_iter().map(String::from).collect()
}

/// Test proxy configuration by making a request to the AWS API (Polly DescribeVoices) in the configured region.
#[tauri::command]
async fn test_proxy_config() -> Result<(), String> {
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts))
        .await
        .map_err(normalize_aws_error)?;
    check_session(&client).await.map_err(normalize_aws_error)
}

/// Returns the app version string for the About screen. Uses the version from Cargo (updated by
/// Release Please) and appends the Git SHA when built from a repo (dev/local or CI).
#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> String {
    let version = app.package_info().version.to_string();
    if let Some(sha) = option_env!("GIT_SHA") {
        format!("{} ({})", version, sha)
    } else {
        version
    }
}

#[tauri::command]
async fn get_caller_identity(_app: tauri::AppHandle) -> Result<CallerIdentity, String> {
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let sdk_config = load_sdk_config_with_options(Some(&opts)).await?;
    let sts = aws_sdk_sts::Client::new(&sdk_config);
    let resp = sts
        .get_caller_identity()
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(CallerIdentity {
        user_id: resp.user_id().map(String::from).unwrap_or_default(),
        account: resp.account().map(String::from).unwrap_or_default(),
        arn: resp.arn().map(String::from).unwrap_or_default(),
    })
}

#[derive(Debug, Serialize)]
pub struct CallerIdentity {
    pub user_id: String,
    pub account: String,
    pub arn: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionStatus {
    pub name: String,
    pub granted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialsAndPermissionsResult {
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip: Option<String>,
    pub permissions: Vec<PermissionStatus>,
}

/// Core logic for checking credentials and permissions. Used by both Tauri command and CLI.
pub async fn run_check_credentials_and_permissions(
    config: AppConfig,
) -> Result<CredentialsAndPermissionsResult, String> {
    let opts = config_to_credential_options(&config);
    let use_manual = config.aws_use_manual.unwrap_or_else(|| {
        config
            .aws_access_key_id
            .as_ref()
            .is_some_and(|s| !s.is_empty())
            && config
                .aws_secret_access_key
                .as_ref()
                .is_some_and(|s| !s.is_empty())
    });
    let config_source = if use_manual {
        "Manual credentials".to_string()
    } else {
        let dir = config.aws_config_dir.clone().unwrap_or_else(|| {
            aws_config_file::default_aws_config_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Default (~/.aws)".to_string())
        });
        if dir == "Default (~/.aws)" {
            aws_config_file::default_aws_config_dir()
                .map(|p| p.join("config").to_string_lossy().into_owned())
                .unwrap_or_else(|| "~/.aws/config".to_string())
        } else {
            Path::new(&dir)
                .join("config")
                .to_string_lossy()
                .into_owned()
        }
    };
    let region = resolve_region(&opts);

    let mut permissions = vec![
        PermissionStatus {
            name: "Access to list voices".to_string(),
            granted: false,
            hint: Some(
                "Have your administrator add `polly:DescribeVoices` permission.".to_string(),
            ),
        },
        PermissionStatus {
            name: "Access to synthesize speech".to_string(),
            granted: false,
            hint: Some(
                "Have your administrator add `polly:SynthesizeSpeech` permission.".to_string(),
            ),
        },
    ];

    apply_proxy_env_from_config(&config);
    let public_ip = fetch_public_ip().await;

    let sdk_config = match load_sdk_config_with_options(Some(&opts)).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(CredentialsAndPermissionsResult {
                authenticated: false,
                error: Some(normalize_aws_error(&e)),
                config_source: Some(config_source),
                region: Some(region),
                user_id: None,
                account: None,
                arn: None,
                public_ip,
                permissions,
            })
        }
    };

    let sts = aws_sdk_sts::Client::new(&sdk_config);
    let identity = match sts.get_caller_identity().send().await {
        Ok(resp) => {
            let user_id = resp.user_id().map(String::from).unwrap_or_default();
            let account = resp.account().map(String::from).unwrap_or_default();
            let arn = resp.arn().map(String::from).unwrap_or_default();
            Some((user_id, account, arn))
        }
        Err(e) => {
            return Ok(CredentialsAndPermissionsResult {
                authenticated: false,
                error: Some(normalize_aws_error(e.to_string())),
                config_source: Some(config_source),
                region: Some(region),
                user_id: None,
                account: None,
                arn: None,
                public_ip,
                permissions,
            })
        }
    };

    let (user_id, account, arn) = identity.unwrap();

    let polly_client = match build_client_with_options(Some(&opts)).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(CredentialsAndPermissionsResult {
                authenticated: true,
                error: Some(normalize_aws_error(&e)),
                config_source: Some(config_source),
                region: Some(region),
                user_id: Some(user_id),
                account: Some(account.clone()),
                arn: Some(arn.clone()),
                public_ip,
                permissions,
            })
        }
    };

    permissions[0].granted = check_session(&polly_client).await.is_ok();
    permissions[1].granted = test_synthesize_speech(&polly_client).await.is_ok();

    Ok(CredentialsAndPermissionsResult {
        authenticated: true,
        error: None,
        config_source: Some(config_source),
        region: Some(region),
        user_id: Some(user_id),
        account: Some(account),
        arn: Some(arn),
        public_ip,
        permissions,
    })
}

#[tauri::command]
async fn check_credentials_and_permissions() -> Result<CredentialsAndPermissionsResult, String> {
    let config = load_app_config(None)?;
    run_check_credentials_and_permissions(config).await
}

#[tauri::command]
async fn polly_check_session(_app: tauri::AppHandle) -> Result<(), String> {
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts))
        .await
        .map_err(normalize_aws_error)?;
    check_session(&client).await.map_err(normalize_aws_error)
}

#[tauri::command]
async fn polly_describe_voices(
    _app: tauri::AppHandle,
    language_code: Option<String>,
    engine: Option<String>,
) -> Result<Vec<VoiceInfo>, String> {
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts))
        .await
        .map_err(normalize_aws_error)?;
    describe_voices(&client, language_code.as_deref(), engine.as_deref())
        .await
        .map_err(normalize_aws_error)
}

#[derive(Clone, serde::Serialize)]
struct GenerateProgressPayload {
    prompt_name: String,
    current: usize,
    total: usize,
    step: String,
}

struct GenerateProgressReporter {
    tx: mpsc::UnboundedSender<GenerateProgressPayload>,
    prompt_name: String,
    current: usize,
    total: usize,
}

impl ProgressReporter for GenerateProgressReporter {
    fn report(&self, step: &str) {
        let _ = self.tx.send(GenerateProgressPayload {
            prompt_name: self.prompt_name.clone(),
            current: self.current,
            total: self.total,
            step: step.to_string(),
        });
    }
}

#[tauri::command]
async fn polly_synthesize_line(
    _app: tauri::AppHandle,
    text: String,
    voice_id: String,
    engine: Option<String>,
    output_dir: String,
) -> Result<String, String> {
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts))
        .await
        .map_err(normalize_aws_error)?;
    let name = format_prompt_filename(&text, config.prompt_file_name_format.as_deref());
    let ext = "ogg";
    let path = std::path::Path::new(&output_dir).join(format!("{}.{}", name, ext));
    synthesize_line(&client, &text, &voice_id, engine.as_deref(), &path, None)
        .await
        .map_err(normalize_aws_error)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
async fn polly_generate_prompts(
    app: tauri::AppHandle,
    lines: Vec<String>,
    voice_id: String,
    engine: Option<String>,
    output_dir: String,
    preset_name: Option<String>,
) -> Result<Vec<String>, String> {
    let preset = preset_name
        .as_deref()
        .and_then(|n| {
            OutputPreset::builtins().into_iter().find(|p| {
                p.name.eq_ignore_ascii_case(n)
                    || n.replace(' ', "-")
                        .eq_ignore_ascii_case(&p.name.replace(' ', "-"))
            })
        })
        .unwrap_or_else(OutputPreset::ogg_only);
    let output_path = std::path::Path::new(&output_dir);
    let config = load_app_config(None)?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts))
        .await
        .map_err(normalize_aws_error)?;
    let filtered: Vec<&str> = lines
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let total = filtered.len();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let app_emit = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(payload) = rx.recv().await {
            let _ = app_emit.emit("generate-progress", &payload);
        }
    });

    let mut paths = Vec::new();
    for (i, trimmed) in filtered.into_iter().enumerate() {
        let current = i + 1;
        let name = format_prompt_filename(trimmed, config.prompt_file_name_format.as_deref());
        let ogg_path = output_path.join(format!("{}.ogg", name));
        let reporter = GenerateProgressReporter {
            tx: tx.clone(),
            prompt_name: name.clone(),
            current,
            total,
        };
        synthesize_line(
            &client,
            trimmed,
            &voice_id,
            engine.as_deref(),
            &ogg_path,
            Some(&reporter),
        )
        .await
        .map_err(normalize_aws_error)?;
        let final_path = apply_preset(&ogg_path, &preset, output_path, Some(&reporter)).await?;
        paths.push(final_path.to_string_lossy().into_owned());
        if preset.format == OutputFormat::Wav && ogg_path.exists() {
            let _ = tokio::fs::remove_file(&ogg_path).await;
        }
    }
    drop(tx);
    Ok(paths)
}

/// Return paths that would be overwritten by generate (i.e. destination paths that already exist).
#[tauri::command]
async fn check_destination_paths(
    lines: Vec<String>,
    output_dir: String,
    preset_name: Option<String>,
) -> Result<Vec<String>, String> {
    let preset = preset_name
        .as_deref()
        .and_then(|n| {
            OutputPreset::builtins().into_iter().find(|p| {
                p.name.eq_ignore_ascii_case(n)
                    || n.replace(' ', "-")
                        .eq_ignore_ascii_case(&p.name.replace(' ', "-"))
            })
        })
        .unwrap_or_else(OutputPreset::ogg_only);
    let output_path = std::path::Path::new(&output_dir);
    let config = load_app_config(None)?;
    let format_opt = config.prompt_file_name_format.as_deref();
    let mut existing = Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let name = format_prompt_filename(trimmed, format_opt);
        let path = match preset.format {
            OutputFormat::Wav => output_path.join(format!("{}.wav", name)),
            OutputFormat::Ogg => output_path.join(format!("{}.ogg", name)),
        };
        if path.exists() {
            if let Some(s) = path.to_str() {
                existing.push(s.to_string());
            }
        }
    }
    Ok(existing)
}

fn main() {
    // Avoid EGL_BAD_PARAMETER crash on Linux (e.g. AppImage on some Wayland/Mesa setups).
    // WebKitGTK tries to create an EGL display; when that fails we get a blank white window and abort.
    // Forcing CPU rendering avoids EGL entirely. Users can set WEBKIT_SKIA_ENABLE_CPU_RENDERING=0 to try GPU.
    #[cfg(target_os = "linux")]
    if std::env::var("WEBKIT_SKIA_ENABLE_CPU_RENDERING").is_err() {
        std::env::set_var("WEBKIT_SKIA_ENABLE_CPU_RENDERING", "1");
    }

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let ran_cli = rt.block_on(cli::run_cli());
    match ran_cli {
        Ok(true) => return,
        Ok(false) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_app_version,
            list_presets,
            get_config,
            save_config,
            delete_config_file,
            get_default_aws_config_dir,
            list_aws_profiles,
            get_aws_profile_env,
            get_profile_region,
            list_aws_regions,
            get_caller_identity,
            check_credentials_and_permissions,
            test_proxy_config,
            check_destination_paths,
            polly_check_session,
            polly_describe_voices,
            polly_synthesize_line,
            polly_generate_prompts,
            get_system_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
