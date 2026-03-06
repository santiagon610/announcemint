//! CLI for announcemint: generate, list-presets, list-voices, check-credentials, test-proxy.

use crate::app_config::{load_app_config, config_to_credential_options};
use crate::polly::{
    build_client_with_options, describe_voices, format_prompt_filename, synthesize_line,
};
use crate::preset::{apply_preset, OutputFormat, OutputPreset};
use crate::{run_check_credentials_and_permissions, CredentialsAndPermissionsResult};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "announcemint")]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    Generate(GenerateArgs),
    ListPresets(ListPresetsArgs),
    ListVoices(ListVoicesArgs),
    CheckCredentials(CheckCredentialsArgs),
    TestProxy(TestProxyArgs),
}

#[derive(Parser, Debug)]
#[command(name = "generate", about = "Generate voice prompts from text (one per line)")]
pub struct GenerateArgs {
    #[arg(long, short = 'o', env = "ANNOUNCEMINT_OUTPUT_DIR")]
    pub output_dir: PathBuf,

    #[arg(long, short = 't')]
    pub text: Option<String>,

    #[arg(long, short = 'f')]
    pub file: Option<PathBuf>,

    #[arg(long, env = "ANNOUNCEMINT_VOICE_ID", default_value = "Joanna")]
    pub voice_id: String,

    #[arg(long, env = "ANNOUNCEMINT_ENGINE")]
    pub engine: Option<String>,

    #[arg(long, env = "ANNOUNCEMINT_LANGUAGE_CODE")]
    pub language_code: Option<String>,

    #[arg(long, short = 'p', env = "ANNOUNCEMINT_PRESET")]
    pub preset: Option<String>,

    #[arg(long, env = "ANNOUNCEMINT_PROMPT_FILE_NAME_FORMAT")]
    pub prompt_file_name_format: Option<String>,

    #[arg(long)]
    pub config_file: Option<PathBuf>,

    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
#[command(name = "list-presets", about = "List built-in output presets")]
pub struct ListPresetsArgs {
    #[arg(long, default_value = "table")]
    pub output: String,
}

#[derive(Parser, Debug)]
#[command(name = "list-voices", about = "List available Polly voices")]
pub struct ListVoicesArgs {
    #[arg(long)]
    pub language_code: Option<String>,

    #[arg(long)]
    pub engine: Option<String>,

    #[arg(long)]
    pub config_file: Option<PathBuf>,

    #[arg(long, default_value = "table")]
    pub output: String,
}

#[derive(Parser, Debug)]
#[command(name = "check-credentials", about = "Check AWS credentials and Polly permissions")]
pub struct CheckCredentialsArgs {
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    #[arg(long, default_value = "table")]
    pub output: String,
}

#[derive(Parser, Debug)]
#[command(name = "test-proxy", about = "Test proxy configuration with Polly")]
pub struct TestProxyArgs {
    #[arg(long)]
    pub config_file: Option<PathBuf>,
}

fn read_lines(args: &GenerateArgs) -> Result<Vec<String>, String> {
    if let Some(ref text) = args.text {
        return Ok(text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }
    if let Some(ref path) = args.file {
        let s = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        return Ok(s
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }
    Err("Provide either --text or --file".to_string())
}

fn merge_config_with_generate_args(
    config: Option<&crate::app_config::AppConfig>,
    args: &GenerateArgs,
) -> (String, Option<String>, Option<String>, Option<String>, Option<String>) {
    let voice_id = args.voice_id.clone();
    let engine = args
        .engine
        .clone()
        .or_else(|| config.and_then(|c| c.engine.clone()));
    let language_code = args
        .language_code
        .clone()
        .or_else(|| config.and_then(|c| c.language_code.clone()));
    let preset = args
        .preset
        .clone()
        .or_else(|| config.and_then(|c| c.preset_name.clone()));
    let prompt_file_name_format = args
        .prompt_file_name_format
        .clone()
        .or_else(|| config.and_then(|c| c.prompt_file_name_format.clone()));
    (
        voice_id,
        engine,
        language_code,
        preset,
        prompt_file_name_format,
    )
}

async fn run_generate(args: GenerateArgs) -> Result<(), String> {
    let lines = read_lines(&args)?;
    if lines.is_empty() {
        return Err("No prompts to generate".to_string());
    }
    let config = args
        .config_file
        .as_deref()
        .and_then(|p| load_app_config(Some(p)).ok())
        .or_else(|| load_app_config(None).ok());
    let (voice_id, engine, _language_code, preset_name, prompt_file_name_format) =
        merge_config_with_generate_args(config.as_ref(), &args);

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

    let output_path = &args.output_dir;

    if args.dry_run {
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let name = format_prompt_filename(trimmed, prompt_file_name_format.as_deref());
            let path = match preset.format {
                OutputFormat::Wav => output_path.join(format!("{}.wav", name)),
                OutputFormat::Ogg => output_path.join(format!("{}.ogg", name)),
            };
            println!("{}", path.display());
        }
        return Ok(());
    }

    let opts = config
        .as_ref()
        .map(config_to_credential_options)
        .unwrap_or_default();
    let client = build_client_with_options(Some(&opts)).await?;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let name = format_prompt_filename(trimmed, prompt_file_name_format.as_deref());
        let ogg_path = output_path.join(format!("{}.ogg", name));
        synthesize_line(
            &client,
            trimmed,
            &voice_id,
            engine.as_deref(),
            &ogg_path,
            None,
        )
        .await?;
        let _ = apply_preset(&ogg_path, &preset, output_path, None).await?;
        if preset.format == OutputFormat::Wav && ogg_path.exists() {
            let _ = tokio::fs::remove_file(&ogg_path).await;
        }
    }
    Ok(())
}

fn run_list_presets(args: ListPresetsArgs) -> Result<(), String> {
    let presets = OutputPreset::builtins();
    if args.output.eq_ignore_ascii_case("json") {
        println!("{}", serde_json::to_string_pretty(&presets).map_err(|e| e.to_string())?);
    } else {
        println!("{:<45} {:<6} {:>8} {:>4}", "Name", "Format", "Rate", "Ch");
        for p in &presets {
            let format_str = match p.format {
                OutputFormat::Ogg => "ogg",
                OutputFormat::Wav => "wav",
            };
            println!(
                "{:<45} {:<6} {:>8} {:>4}",
                p.name,
                format_str,
                format!("{} Hz", p.sample_rate),
                p.channels
            );
        }
    }
    Ok(())
}

async fn run_list_voices(args: ListVoicesArgs) -> Result<(), String> {
    let config = args
        .config_file
        .as_deref()
        .and_then(|p| load_app_config(Some(p)).ok())
        .or_else(|| load_app_config(None).ok());
    let opts = config
        .as_ref()
        .map(config_to_credential_options)
        .unwrap_or_default();
    let client = build_client_with_options(Some(&opts)).await?;
    let voices =
        describe_voices(&client, args.language_code.as_deref(), args.engine.as_deref()).await?;
    if args.output.eq_ignore_ascii_case("json") {
        println!("{}", serde_json::to_string_pretty(&voices).map_err(|e| e.to_string())?);
    } else {
        println!("{:<20} {:<12} {:<8}", "Id", "Language", "Engines");
        for v in &voices {
            let engines = v.supported_engines.join(", ");
            println!("{:<20} {:<12} {}", v.id, v.language_code, engines);
        }
    }
    Ok(())
}

fn print_credentials_result(result: &CredentialsAndPermissionsResult, json: bool) -> Result<(), String> {
    if json {
        println!("{}", serde_json::to_string_pretty(result).map_err(|e| e.to_string())?);
    } else {
        if let Some(ref src) = result.config_source {
            println!("Config source: {}", src);
        }
        if let Some(ref r) = result.region {
            println!("Region: {}", r);
        }
        println!("Authenticated: {}", result.authenticated);
        if let Some(ref e) = result.error {
            println!("Error: {}", e);
        }
        if let Some(ref uid) = result.user_id {
            println!("User ID: {}", uid);
        }
        if let Some(ref acc) = result.account {
            println!("Account: {}", acc);
        }
        if let Some(ref arn) = result.arn {
            println!("ARN: {}", arn);
        }
        if let Some(ref ip) = result.public_ip {
            println!("Public IP: {}", ip);
        }
        println!("Permissions:");
        for p in &result.permissions {
            let ok = if p.granted { "yes" } else { "no" };
            println!("  {}: {}", p.name, ok);
            if let Some(ref h) = p.hint {
                println!("    ({})", h);
            }
        }
    }
    Ok(())
}

async fn run_check_credentials(args: CheckCredentialsArgs) -> Result<(), String> {
    let config = args
        .config_file
        .as_deref()
        .and_then(|p| load_app_config(Some(p)).ok())
        .or_else(|| load_app_config(None).ok())
        .ok_or_else(|| "Failed to load config".to_string())?;
    let result = run_check_credentials_and_permissions(config).await?;
    print_credentials_result(&result, args.output.eq_ignore_ascii_case("json"))?;
    Ok(())
}

async fn run_test_proxy(args: TestProxyArgs) -> Result<(), String> {
    let config = args
        .config_file
        .as_deref()
        .and_then(|p| load_app_config(Some(p)).ok())
        .or_else(|| load_app_config(None).ok())
        .ok_or_else(|| "Failed to load config".to_string())?;
    let opts = config_to_credential_options(&config);
    let client = build_client_with_options(Some(&opts)).await?;
    crate::polly::check_session(&client).await.map_err(|e| e.to_string())?;
    println!("Proxy test OK");
    Ok(())
}

/// Returns Ok(true) if a CLI subcommand was run (caller should exit), Ok(false) if no subcommand, Err on failure.
pub async fn run_cli() -> Result<bool, String> {
    let cli = Cli::parse();
    let ran = match cli.subcommand {
        Some(Subcommand::Generate(args)) => {
            run_generate(args).await?;
            true
        }
        Some(Subcommand::ListPresets(args)) => {
            run_list_presets(args)?;
            true
        }
        Some(Subcommand::ListVoices(args)) => {
            run_list_voices(args).await?;
            true
        }
        Some(Subcommand::CheckCredentials(args)) => {
            run_check_credentials(args).await?;
            true
        }
        Some(Subcommand::TestProxy(args)) => {
            run_test_proxy(args).await?;
            true
        }
        None => false,
    };
    Ok(ran)
}
