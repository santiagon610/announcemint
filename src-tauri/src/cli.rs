//! CLI entry point for `announcemint generate`: parse args (with env overrides) and run generation.

use crate::polly::{build_client, sanitize_filename, synthesize_line};
use crate::preset::{apply_preset, OutputFormat, OutputPreset};
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
}

#[derive(Parser, Debug)]
#[command(name = "generate")]
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

/// Returns Ok(true) if generate subcommand was run (caller should exit), Ok(false) if no subcommand, Err on failure.
pub async fn run_generate() -> Result<bool, String> {
    let cli = Cli::parse();
    let Some(Subcommand::Generate(args)) = cli.subcommand else {
        return Ok(false);
    };
    let lines = read_lines(&args)?;
    if lines.is_empty() {
        return Err("No prompts to generate".to_string());
    }
    let preset = args
        .preset
        .as_deref()
        .and_then(|n| {
            OutputPreset::builtins().into_iter().find(|p| {
                p.name.eq_ignore_ascii_case(n)
                    || n.replace(' ', "-")
                        .eq_ignore_ascii_case(&p.name.replace(' ', "-"))
            })
        })
        .unwrap_or_else(OutputPreset::ogg_only);
    let region = std::env::var("AWS_REGION").ok();
    let client = build_client(region.as_deref()).await?;
    let output_path = &args.output_dir;
    for line in &lines {
        let name = sanitize_filename(line);
        let ogg_path = output_path.join(format!("{}.ogg", name));
        synthesize_line(
            &client,
            line,
            &args.voice_id,
            args.engine.as_deref(),
            &ogg_path,
            None,
        )
        .await?;
        let _ = apply_preset(&ogg_path, &preset, output_path, None).await?;
        if preset.format == OutputFormat::Wav && ogg_path.exists() {
            let _ = tokio::fs::remove_file(&ogg_path).await;
        }
    }
    Ok(true)
}
