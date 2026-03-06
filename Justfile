# Announcemint – just recipes
# https://github.com/casey/just

# Run all application tests (Rust unit + integration)
test:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run the desktop GUI locally (dev server + Tauri window).
# Build Rust first so the app window opens only after the backend is ready.
gui:
    cargo build --manifest-path src-tauri/Cargo.toml
    npm run tauri dev

# Run the CLI. Pass arguments after the recipe, e.g.:
#   just cli generate --output-dir ./out --text "Hello"
#   just cli generate --output-dir ./out --file prompts.txt --preset "Two-Way Voice Prompt"
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
