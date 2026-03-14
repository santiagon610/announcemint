# Announcemint – just recipes
# https://github.com/casey/just

# Install Node dependencies. Run after clone or `just cleanup` before `just gui`.
install:
    npm install

# Run all application tests (Rust unit + integration)
test:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run the desktop GUI locally (dev server + Tauri window).
# Build Rust first so the app window opens only after the backend is ready.
# On Linux, if the build fails with missing webkit2gtk/libjavascriptcore/libsoup, run: just gui-deps
gui:
    cargo build --manifest-path src-tauri/Cargo.toml
    npm run tauri dev

# Print Linux system packages required for the GUI (Fedora). See README for other distros and PKG_CONFIG_PATH.
gui-deps:
    @echo "On Fedora/RHEL, install WebKitGTK + GLib/GTK + librsvg (for AppImage/linuxdeploy):"
    @echo "  sudo dnf install glib2-devel webkit2gtk4.1-devel libsoup3-devel javascriptcoregtk4.1-devel librsvg2-devel"
    @echo "If you use Homebrew on Linux, also: export PKG_CONFIG_PATH=\"/usr/lib64/pkgconfig:$$PKG_CONFIG_PATH\""

gui-build-macos:
    cargo build --manifest-path src-tauri/Cargo.toml
    npm run tauri build -- --bundles app

# Build Linux AppImage (gzip compression + EGL workaround via repack). Output: src-tauri/target/release/bundle/appimage/*.AppImage
# APPIMAGE_EXTRACT_AND_RUN=1: linuxdeploy AppImage extract-and-run (avoids FUSE "failed to run linuxdeploy").
# NO_STRIP=true: avoid strip failing on .relr.dyn (modern glibc) on Fedora/rolling distros.
# --verbose: required on some hosts so linuxdeploy runs correctly (see tauri-apps/tauri#14796).
appimage:
    APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=true npm run tauri build -- --bundles appimage --verbose
    chmod +x scripts/repack-appimage-gzip.sh
    sh -c 'for f in src-tauri/target/release/bundle/appimage/*.AppImage; do ./scripts/repack-appimage-gzip.sh "$f"; done'

# Run the CLI. Uses same config as GUI when present. Examples:
#   just cli generate --output-dir ./out --text "Hello"
#   just cli generate --output-dir ./out --file prompts.txt --preset "WAV: Two-Way Radio Voice Prompt"
#   just cli list-presets
#   just cli list-voices
#   just cli check-credentials
#   just cli test-proxy
cli *ARGS:
    cargo run --manifest-path src-tauri/Cargo.toml -- {{ARGS}}

# Build the CLI release binary (for faster repeated CLI runs)
build-cli:
    cargo build --release --manifest-path src-tauri/Cargo.toml

# Run the release CLI binary (use after `just build-cli` for faster runs)
cli-release *ARGS:
    ./src-tauri/target/release/announcemint {{ARGS}}

cleanup:
    rm -rfv dist/ node_modules/ package-lock.json

fmt:
    cargo fmt --all --manifest-path src-tauri/Cargo.toml
    npm run format
