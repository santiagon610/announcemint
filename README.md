# Announcemint

Convert text to speech using **AWS Polly**, with presets for two-way radio voice prompts and more. Cross-platform desktop app and CLI: one prompt per line, Ogg Vorbis or in-app conversion to WAV.

App name, publisher, docs URL, and GitHub repo are set in **`brand.json`** at the project root so you can rebrand in one place (see [Rebranding](#rebranding)).

### Getting the app

Prebuilt installers (Windows MSI, macOS .app, Linux AppImage) are published on the [Releases](https://github.com/hlvtechnologies/announcemint/releases) page. You need [AWS credentials](#requirements) with Polly permissions before generating speech.

## Features

### Desktop app

- **Main screen**: Prompts text area (one per line), output directory, **Generate Prompts** button. Progress shows the current prompt name, a progress bar, and step-by-step status (Submitted to Polly → Downloading → Converting → Saving). If any destination files already exist, the app asks for overwrite confirmation before generating.
- **Status indicator**: Green / yellow / red dot next to Settings shows AWS auth and permission status (authenticated + Polly permissions, authenticated only, or not authenticated).
- **Help**: Opens the docs URL in the system browser (set in `brand.json`).
- **About**: App version (including Git SHA for dev builds), license, and project links.

### Settings (full-page, expandable drawers)

- **Network Proxy**: Optional HTTP/HTTPS/SOCKS proxy (protocol, host, port, username, password). **Test proxy** checks that the app can reach the AWS API through the proxy.
- **AWS Account**: Credential source (AWS config file or manual keys). Config directory and profile selection; **Check credentials and permissions** validates auth and reports User ID, Account, ARN, Region, config file path, public IP, and whether `polly:DescribeVoices` and `polly:SynthesizeSpeech` are granted.
- **Voice Options**: Language (including system locale), engine (standard/neural), voice, output preset (e.g. OGG Vorbis, WAV: Two-Way Radio Voice Prompt). Option to remember prompt names after closing.
- **Saving Prompts**: Output directory and **Filename Formatting** (e.g. hyphens, underscores, lower/upper case).

Configuration is stored in a platform-specific user config directory (e.g. `~/.config/<publisher>/<appName>/config.json`), where `publisher` and `appName` come from `brand.json`.

### Output presets

- **OGG Vorbis**: Polly output as-is.
- **WAV: Two-Way Radio Voice Prompt**: 8 kHz, 16-bit, mono, little endian. In-app Ogg→WAV conversion (no SoX).

### CLI

Same generation logic as the app. Prompts from `--file` or `--text`; output directory and optional voice, engine, preset.

```bash
# Build first (or use cargo run)
cargo build --release --manifest-path src-tauri/Cargo.toml

# Generate
./src-tauri/target/release/announcemint generate --output-dir ./out --file prompts.txt --preset "WAV: Two-Way Radio Voice Prompt"
```

**Options**: `--output-dir` / `-o`, `--file` / `-f`, `--text` / `-t`, `--voice-id`, `--engine`, `--preset` / `-p`.  
**Environment**: `ANNOUNCEMINT_OUTPUT_DIR`, `ANNOUNCEMINT_VOICE_ID`, `ANNOUNCEMINT_ENGINE`, `ANNOUNCEMINT_LANGUAGE_CODE`, `ANNOUNCEMINT_PRESET`, plus standard AWS (`AWS_PROFILE`, `AWS_REGION`, etc.).

## Requirements

- **Node.js** and **npm** (for the frontend and Tauri dev/build)
- **Rust** (for the backend and CLI)
- **AWS credentials** with `polly:DescribeVoices` and `polly:SynthesizeSpeech` (env vars, `~/.aws` config, or manual entry in Settings)

### Linux: system libraries for the GUI

The desktop app uses WebKitGTK. Install the development packages so `pkg-config` can find them (e.g. for `just gui` or `npm run tauri dev`).

- **Fedora / RHEL**:
  ```bash
  sudo dnf install glib2-devel webkit2gtk4.1-devel libsoup3-devel librsvg2-devel
  ```
  (On some Fedora versions you may need `javascriptcoregtk4.1-devel` if the above is not enough. If you see `gio-2.0` or other GLib `.pc` not found, install `glib2-devel`. For AppImage builds, `librsvg2-devel` is required by the linuxdeploy GTK plugin.)

- **Debian / Ubuntu**:
  ```bash
  sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
  ```

If the build still reports a missing `.pc` file:

- **Using Linuxbrew or Homebrew on Linux**: That `pkg-config` only searches Homebrew’s prefix, so it never sees system libraries. Prepend the system pkg-config directory so the WebKit/GTK `.pc` files are found:
  ```bash
  export PKG_CONFIG_PATH="/usr/lib64/pkgconfig:$PKG_CONFIG_PATH"
  ```
  Then run `just gui` (or `npm run tauri dev`) in the same shell. You can add that line to your shell profile if you use the GUI often.

- Otherwise, set `PKG_CONFIG_PATH` to the directory that contains the `.pc` file, or install the matching `-devel` / `-dev` package for that library.

### Linux: dark mode and window theme

On Linux, Tauri does not yet reliably propagate the system theme to the WebView or native window (see [tauri#9427](https://github.com/tauri-apps/tauri/issues/9427)). The app works around this by reading the **XDG Settings portal** (`org.freedesktop.appearance.color-scheme`) and applying dark/light to the in-app UI. So the **content** of the window should follow your system dark/light preference. The **window border and title bar** are drawn by the desktop and may stay light until Tauri/tao adds full Linux theme support; there is no theme option in `tauri.conf.json` for Linux (it is only implemented on Windows and macOS).

## Development

### Bootstrap (first-time setup)

Before running the GUI or building the app, set up the repo once:

1. **Install dependencies** (Node and Tauri CLI):
   ```bash
   npm install
   ```
   If you see `tauri: command not found` when running `just gui`, the Tauri CLI is missing — run `npm install` from the project root.

2. **Create the frontend build output** so Tauri can load the UI. The config expects `dist/` at the project root. Either build the frontend once:
   ```bash
   npm run build
   ```
   or create an empty directory (the app will start but the window will have no UI until you run `npm run build`):
   ```bash
   mkdir -p dist
   ```

3. On **Linux**, install [system libraries for the GUI](#linux-system-libraries-for-the-gui) and, if you use Linuxbrew, set `PKG_CONFIG_PATH` as described there.

Then you can run `just gui` or the commands below.

### Daily workflow

- **GUI**: `just gui` or `npm run tauri dev` (Rust is built first so the window opens when the backend is ready).
- **CLI**: `just cli generate --output-dir ./out --text "Hello"` or `just cli generate --output-dir ./out --file prompts.txt`.
- **Tests**: `just test` or `cargo test --manifest-path src-tauri/Cargo.toml`.

Optional: use the **Tauri Development** launch config in VS Code (Run and Debug) to run the GUI.

### Common issues

- **`just gui` fails with "webkit2gtk-4.1", "libsoup-3.0", or "javascriptcoregtk-4.1" not found**  
  You need the Linux system libraries for the GUI. See [Linux: system libraries for the GUI](#linux-system-libraries-for-the-gui). On Fedora you can run `just gui-deps` to print the install command.

- **`frontendDist` configuration is set to `"../dist"` but this path doesn't exist**  
  Tauri requires the `dist/` directory (built frontend). Run `npm run build` once to create it, or `mkdir -p dist` for an empty placeholder.

- **`tauri: command not found`**  
  The Tauri CLI is provided by npm. Run `npm install` in the project root so `node_modules/.bin` includes it; then use `just gui` or `npm run tauri dev` (do not run `tauri` directly unless it’s on your PATH).

- **Blank window on first open (dev)**  
  The window can open before the Vite dev server is ready. Reload the window (e.g. Ctrl+R / Cmd+R) or start the dev server first (`npm run dev`), then run `just gui` in another terminal.

- **Title bar**  
  The native title bar is drawn by the OS and does not follow the app’s CSS or dark mode. To have the title bar match the app, you’d need to set `decorations: false` and implement a custom HTML/CSS title bar with a drag region (and optionally window controls via the Tauri window API).

## Build

- **Windows (MSI)**: On Windows with WiX installed, `npm run tauri build` produces an MSI in `src-tauri/target/release/bundle/msi/`.
- **macOS (.app)**: On macOS, `npm run tauri build` produces the app bundle (e.g. in `src-tauri/target/release/bundle/macos/`).
- **Linux**: `npm run tauri build` produces .deb and AppImage. In environments without FUSE (e.g. Docker, CI), set `APPIMAGE_EXTRACT_AND_RUN=1` so the linuxdeploy AppImage can run. If you see “Squashfs image uses (null) compression” when launching via AppImageLauncher, run `./scripts/repack-appimage-gzip.sh path/to/foo.AppImage` or run the AppImage directly. Release builds are repacked with gzip so AppImageLauncher can read them. If you get "Could not create default EGL display: EGL_BAD_PARAMETER" and a blank window: the repack step (used in CI/releases) injects `WEBKIT_SKIA_ENABLE_CPU_RENDERING=1` into the AppImage’s AppRun so the variable is set before the app starts; the in-app fallback only helps when not using the AppImage. To try GPU, run with `WEBKIT_SKIA_ENABLE_CPU_RENDERING=0` before launching.

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
