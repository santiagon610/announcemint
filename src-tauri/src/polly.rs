//! AWS Polly client: session check, describe voices, synthesize speech to Ogg.

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use aws_sdk_polly::types::{Engine, OutputFormat, TextType, VoiceId};
use aws_sdk_polly::Client;
use aws_types::SdkConfig;
use std::path::Path;
use std::str::FromStr;

/// Options for building an AWS client (profile, config dir, manual credentials, proxy).
#[derive(Clone, Debug, Default)]
pub struct AwsCredentialOptions {
    pub profile: Option<String>,
    pub config_dir: Option<String>,
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub proxy_enabled: bool,
    pub proxy_url: Option<String>,
}

pub fn resolve_region(opts: &AwsCredentialOptions) -> String {
    opts.region
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("AWS_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string())
}

/// Load SDK config from optional credential options (for use by Polly, STS, etc.).
pub async fn load_sdk_config_with_options(
    opts: Option<&AwsCredentialOptions>,
) -> Result<SdkConfig, String> {
    if let Some(o) = opts {
        if o.proxy_enabled {
            if let Some(url) = o.proxy_url.as_ref().filter(|s| !s.is_empty()) {
                std::env::set_var("HTTP_PROXY", url.as_str());
                std::env::set_var("HTTPS_PROXY", url.as_str());
            }
        }
        // When proxy is disabled we do not remove HTTP_PROXY/HTTPS_PROXY so the process
        // can still use the environment's proxy (e.g. terminal with proxy set).
    }

    let region = opts
        .map(resolve_region)
        .unwrap_or_else(|| std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()));
    let region = aws_config::Region::new(region);

    if let Some(o) = opts {
        let use_manual = o.access_key_id.as_ref().is_some_and(|s| !s.is_empty())
            && o.secret_access_key.as_ref().is_some_and(|s| !s.is_empty());
        if use_manual {
            let creds = Credentials::new(
                o.access_key_id.as_deref().unwrap_or(""),
                o.secret_access_key.as_deref().unwrap_or(""),
                None,
                None,
                "manual",
            );
            return Ok(aws_config::defaults(BehaviorVersion::latest())
                .region(region)
                .credentials_provider(creds)
                .load()
                .await);
        }
        if let Some(profile) = o.profile.as_deref().filter(|s| !s.is_empty()) {
            let config_dir_path = o
                .config_dir
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(Path::new);
            let profile_region =
                crate::aws_config_file::get_profile_region(config_dir_path, profile)
                    .or_else(|| std::env::var("AWS_REGION").ok())
                    .unwrap_or_else(|| "us-east-1".to_string());
            let region = aws_config::Region::new(profile_region);
            let mut loader = aws_config::defaults(BehaviorVersion::latest())
                .region(region.clone())
                .profile_name(profile);
            if let Some(dir) = o.config_dir.as_ref().filter(|s| !s.is_empty()) {
                let cred_path = Path::new(dir).join("credentials");
                let config_path = Path::new(dir).join("config");
                let mut builder =
                    EnvConfigFiles::builder().with_file(EnvConfigFileKind::Config, config_path);
                if cred_path.exists() {
                    builder = builder.with_file(EnvConfigFileKind::Credentials, cred_path);
                }
                loader = loader.profile_files(builder.build());
            }
            return Ok(loader.load().await);
        }
    }

    Ok(aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .load()
        .await)
}

/// Build a Polly client from optional credential options.
pub async fn build_client_with_options(
    opts: Option<&AwsCredentialOptions>,
) -> Result<Client, String> {
    let config = load_sdk_config_with_options(opts).await?;
    Ok(Client::new(&config))
}

/// Build a Polly client using the default credential chain (env, ~/.aws/credentials, etc.)
/// and optional region (defaults to AWS_REGION or us-east-1).
#[allow(dead_code)]
pub async fn build_client(region_override: Option<&str>) -> Result<Client, String> {
    let opts = AwsCredentialOptions {
        region: region_override.map(String::from),
        ..Default::default()
    };
    build_client_with_options(Some(&opts)).await
}

/// Check that we have a valid AWS session by calling DescribeVoices (lightweight).
pub async fn check_session(client: &Client) -> Result<(), String> {
    client
        .describe_voices()
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Minimal SynthesizeSpeech call to verify polly:SynthesizeSpeech permission.
pub async fn test_synthesize_speech(client: &Client) -> Result<(), String> {
    let resp = client
        .synthesize_speech()
        .output_format(OutputFormat::OggVorbis)
        .text("Hi")
        .voice_id(VoiceId::Joanna)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let _ = resp
        .audio_stream
        .collect()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Voice info returned to the frontend for the config UI.
#[derive(serde::Serialize, Clone, Debug)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language_code: String,
    pub language_name: Option<String>,
    pub gender: Option<String>,
    pub supported_engines: Vec<String>,
}

/// List voices, optionally filtered by language and engine.
pub async fn describe_voices(
    client: &Client,
    language_code: Option<&str>,
    engine: Option<&str>,
) -> Result<Vec<VoiceInfo>, String> {
    let mut req = client.describe_voices();
    if let Some(lc) = language_code {
        req = req.set_language_code(Some(
            aws_sdk_polly::types::LanguageCode::from_str(lc)
                .map_err(|_| format!("invalid language code: {}", lc))?,
        ));
    }
    if let Some(eng) = engine {
        let e = match eng {
            "neural" => Engine::Neural,
            "standard" => Engine::Standard,
            "long-form" => Engine::LongForm,
            "generative" => Engine::Generative,
            _ => Engine::Standard,
        };
        req = req.set_engine(Some(e));
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let voices = resp
        .voices()
        .iter()
        .map(|v| VoiceInfo {
            id: v.id().map(|s| s.to_string()).unwrap_or_default(),
            name: v.name().map(|s| s.to_string()).unwrap_or_default(),
            language_code: v
                .language_code()
                .map(|c| c.as_str().to_string())
                .unwrap_or_default(),
            language_name: v.language_name().map(|s| s.to_string()),
            gender: v.gender().map(|g| g.as_str().to_string()),
            supported_engines: v
                .supported_engines()
                .iter()
                .map(|e| e.as_str().to_string())
                .collect(),
        })
        .collect();
    Ok(voices)
}

/// Sanitize a line of text for use as a filename (replace spaces with underscores, remove invalid chars).
#[allow(dead_code)]
pub fn sanitize_filename(text: &str) -> String {
    text.trim()
        .chars()
        .map(|c| {
            if c.is_whitespace() || c == '/' || c == '\\' {
                '_'
            } else {
                c
            }
        })
        .filter(|c| {
            !std::path::Path::new(&c.to_string())
                .components()
                .next()
                .is_some_and(|x| matches!(x, std::path::Component::ParentDir))
        })
        .collect::<String>()
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

/// Format prompt text as a filename stem based on the chosen format.
/// Returns only the stem (no extension). Format one of: "none", "hyphen", "hyphen_lower", "hyphen_upper", "underscore", "underscore_lower", "underscore_upper".
pub fn format_prompt_filename(text: &str, format: Option<&str>) -> String {
    let s = text.trim();
    if s.is_empty() {
        return "prompt".to_string();
    }
    let format = format.unwrap_or("underscore");
    match format {
        "none" => s
            .chars()
            .map(|c| {
                if matches!(
                    c,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'
                ) {
                    '_'
                } else {
                    c
                }
            })
            .collect::<String>(),
        "hyphen" | "hyphen_lower" | "hyphen_upper" => {
            let stem = replace_spaces_and_special(s, '-');
            apply_case(&stem, format)
        }
        "underscore" | "underscore_lower" | "underscore_upper" => {
            let stem = replace_spaces_and_special(s, '_');
            apply_case(&stem, format)
        }
        _ => {
            let stem = replace_spaces_and_special(s, '_');
            apply_case(&stem, format)
        }
    }
}

fn replace_spaces_and_special(s: &str, sep: char) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_sep = true;
    let hyphen_is_sep = sep == '-';
    for c in s.chars() {
        let is_sep = c.is_whitespace()
            || matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'
            )
            || (hyphen_is_sep && c == '-');
        if is_sep {
            if !prev_was_sep {
                result.push(sep);
                prev_was_sep = true;
            }
        } else {
            result.push(c);
            prev_was_sep = false;
        }
    }
    result.trim_matches(sep).to_string()
}

fn apply_case(s: &str, format: &str) -> String {
    match format {
        "hyphen_lower" | "underscore_lower" => s.to_lowercase(),
        "hyphen_upper" | "underscore_upper" => s.to_uppercase(),
        _ => s.to_string(),
    }
}

/// Synthesize one line of text to Ogg Vorbis and write to the given path.
/// Uses SSML with a simple speak wrapper to match the original script behavior.
/// Calls progress with "submitted", "downloading", "saving_ogg" at each step.
pub async fn synthesize_line(
    client: &Client,
    text: &str,
    voice_id: &str,
    engine: Option<&str>,
    output_path: &std::path::Path,
    progress: Option<&dyn crate::progress::ProgressReporter>,
) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty text".into());
    }
    // Match original script: SSML with space-wrapped content
    let ssml = format!("<speak> {} </speak>", text);

    let voice = VoiceId::from_str(voice_id).unwrap_or(VoiceId::Joanna);
    let mut req = client
        .synthesize_speech()
        .output_format(OutputFormat::OggVorbis)
        .text_type(TextType::Ssml)
        .text(ssml)
        .voice_id(voice);

    if let Some(eng) = engine {
        let e = match eng {
            "neural" => Engine::Neural,
            "standard" => Engine::Standard,
            "long-form" => Engine::LongForm,
            "generative" => Engine::Generative,
            _ => Engine::Standard,
        };
        req = req.set_engine(Some(e));
    }

    if let Some(r) = progress {
        r.report("submitted");
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if let Some(r) = progress {
        r.report("downloading");
    }
    let aggregated = resp
        .audio_stream
        .collect()
        .await
        .map_err(|e| e.to_string())?
        .into_bytes();
    tokio::fs::write(output_path, aggregated)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World"), "Hello_World");
        assert_eq!(sanitize_filename("  Tac 2  "), "Tac_2");
        assert_eq!(sanitize_filename("Ops 1, V R S 1"), "Ops_1,_V_R_S_1");
    }
}
