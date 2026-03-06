# Announcemint

Convert text to speech using **AWS Polly**, with presets for two-way radio voice prompts and more. Cross-platform desktop app and CLI: one prompt per line, Ogg Vorbis or in-app conversion to WAV.

App name, publisher, docs URL, and GitHub repo are set in **`brand.json`** at the project root so you can rebrand in one place (see [Rebranding](#rebranding)).

## Features

### Desktop app

- **Main screen**: Prompts text area (one per line), output directory, **Generate Prompts** button. Progress shows the current prompt name, a progress bar, and step-by-step status (Submitted to Polly → Downloading → Converting → Saving). If any destination files already exist, the app asks for overwrite confirmation before generating.
- **Status indicator**: Green / yellow / red dot next to Settings shows AWS auth and permission status (authenticated + Polly permissions, authenticated only, or not authenticated).
- **Help**: Opens the docs URL in the system browser (set in `brand.json`).
- **About**: App version (including Git SHA for dev builds), license, and project links.

### Settings (full-page, expandable drawers)

- **Network Proxy**: Optional HTTP/HTTPS/SOCKS proxy (protocol, host, port, username, password). **Test proxy** checks that the app can reach the AWS API through the proxy.
- **AWS Account**: Credential source (AWS config file or manual keys). Config directory and profile selection; **Check credentials and permissions** validates auth and reports User ID, Account, ARN, Region, config file path, public IP, and whether `polly:DescribeVoices` and `polly:SynthesizeSpeech` are granted.
- **Voice Options**: Language (including system locale), engine (standard/neural), voice, output preset (e.g. OGG Vorbis, Two-Way Voice Prompt WAV). Option to remember prompt names after closing.
- **Saving Prompts**: Output directory and **Filename Formatting** (e.g. hyphens, underscores, lower/upper case).

Configuration is stored in a platform-specific user config directory (e.g. `~/.config/<publisher>/<appName>/config.json`), where `publisher` and `appName` come from `brand.json`.

### Output presets

- **OGG Vorbis**: Polly output as-is.
- **WAV: Two-Way Voice Prompt**: 8 kHz, 16-bit, mono, little endian. In-app Ogg→WAV conversion (no SoX).

### CLI

Same generation logic as the app. Prompts from `--file` or `--text`; output directory and optional voice, engine, preset.

```bash
# Build first (or use cargo run)
cargo build --release --manifest-path src-tauri/Cargo.toml

# Generate
./src-tauri/target/release/announcemint generate --output-dir ./out --file prompts.txt --preset "WAV: Two-Way Voice Prompt"
```

**Options**: `--output-dir` / `-o`, `--file` / `-f`, `--text` / `-t`, `--voice-id`, `--engine`, `--preset` / `-p`.  
**Environment**: `ANNOUNCEMINT_OUTPUT_DIR`, `ANNOUNCEMINT_VOICE_ID`, `ANNOUNCEMINT_ENGINE`, `ANNOUNCEMINT_LANGUAGE_CODE`, `ANNOUNCEMINT_PRESET`, plus standard AWS (`AWS_PROFILE`, `AWS_REGION`, etc.).

## Requirements

- **Node.js** and **npm** (for the frontend and Tauri dev/build)
- **Rust** (for the backend and CLI)
- **AWS credentials** with `polly:DescribeVoices` and `polly:SynthesizeSpeech` (env vars, `~/.aws` config, or manual entry in Settings)

## Development

```bash
npm install
```

- **GUI**: `just gui` or `npm run tauri dev` (Rust is built first so the window opens when the backend is ready).
- **CLI**: `just cli generate --output-dir ./out --text "Hello"` or `just cli generate --output-dir ./out --file prompts.txt`.
- **Tests**: `just test` or `cargo test --manifest-path src-tauri/Cargo.toml`.

Optional: use the **Tauri Development** launch config in VS Code (Run and Debug) to run the GUI.

## Build

- **Windows (MSI)**: On Windows with WiX installed, `npm run tauri build` produces an MSI in `src-tauri/target/release/bundle/msi/`.
- **macOS (.app)**: On macOS, `npm run tauri build` produces the app bundle (e.g. in `src-tauri/target/release/bundle/macos/`).
- **Linux**: `npm run tauri build` produces .deb and AppImage. For Flatpak, use a separate Flatpak manifest and build with `flatpak-builder`.

## Rebranding

Edit **`brand.json`** at the project root:

- **`publisher`** – Used in the config directory path (e.g. `~/.config/<publisher>/<appName>/`).
- **`appName`** – Display name (window title, headers, About). Also used in the config path.
- **`docsUrl`** – URL opened by the Help button; also used for the opener permission in capabilities.
- **`githubRepo`** – URL for the GitHub link on the About screen.
- **`shortDescription`** / **`longDescription`** – Bundle metadata (e.g. for installers).

Then run **`npm run sync-brand`** to update `src-tauri/tauri.conf.json` (product name, window title, identifier, CLI description) and `src-tauri/capabilities/default.json` (docs URL allow list). The build and dev commands run `sync-brand` automatically. Rebuild the app so the Rust backend picks up the new publisher/app name for the config directory.

Optionally update **`package.json`** `name` and the CLI binary name in `src-tauri/src/cli.rs` if you want the npm package and CLI command to match your brand.

## License

GPL-3.0. See [LICENSE](LICENSE).
