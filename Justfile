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
gui:
    cargo build --manifest-path src-tauri/Cargo.toml
    npm run tauri dev

gui-build-macos:
    cargo build --manifest-path src-tauri/Cargo.toml
    npm run tauri build -- --bundles app

# Build Flatpak for Linux (requires flatpak, flatpak-builder, and GNOME runtime).
# Run `flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46` first.
flatpak:
    npm run tauri build -- --bundles deb --ci
    cp src-tauri/target/release/bundle/deb/*.deb flatpak/announcemint.deb
    flatpak-builder --repo=repo --force-clean build-dir flatpak/com.santiagon610.Announcemint.yml
    flatpak build-bundle repo announcemint.flatpak com.santiagon610.Announcemint
    @echo "Built announcemint.flatpak. Install with: flatpak install announcemint.flatpak"

# Run the CLI. Uses same config as GUI when present. Examples:
#   just cli generate --output-dir ./out --text "Hello"
#   just cli generate --output-dir ./out --file prompts.txt --preset "Two-Way Voice Prompt"
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
