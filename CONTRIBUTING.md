# Contributing to Announcemint

Thanks for your interest in contributing. This document covers how to run tests, format and lint code, and open pull requests.

## Getting set up

Follow the [Development](README.md#development) section in the README: install Node and Rust dependencies, create the frontend build (`npm run build` or `mkdir dist`), and on Linux install the [system libraries for the GUI](README.md#linux-system-libraries-for-the-gui). Optional: install [just](https://github.com/casey/just) to use the `just` recipes.

## Running tests

- **All tests**: `just test` or `cargo test --manifest-path src-tauri/Cargo.toml`
- Tests are Rust-only (unit and integration in `src-tauri/`). CI runs these on every push and pull request.

## Code format and lint

Before submitting a pull request, ensure code is formatted and passes lint:

- **Format (Rust + frontend)**  
  `just fmt` — runs `cargo fmt` and `npm run format` (Prettier on `src/**`).
- **Format check only**  
  - Rust: `cargo fmt --all -- --check` in `src-tauri/`
  - Frontend: `npm run format:check`
- **Lint**  
  - Rust: `cargo clippy --all-targets -- -D warnings` in `src-tauri/`
  - Frontend: `npm run lint` (ESLint)

CI runs Rust format check, Clippy, and frontend format check; fix any failures locally first.

## Pull requests

- **Branch**: Open PRs against `main`. Use a feature branch (e.g. `feat/thing` or `fix/issue`).
- **Semantic PR titles**: The CI checks that the PR title follows [Conventional Commits](https://www.conventionalcommits.org/). Use a type prefix and short description, for example:
  - `feat: add preset for narrowband WAV`
  - `fix: proxy test timeout on Windows`
  - `docs: update Linux deps for Ubuntu 24`
  - `refactor: share preset resolution between CLI and GUI`

  Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.

- **Conventional commits**: The project uses [Release Please](https://github.com/googleapis/release-please) for versioning and changelogs. Commits on `main` that follow Conventional Commits (e.g. `feat:`, `fix:`) drive the next release and `CHANGELOG.md`. PR titles are not used for that; the merge commit or squash message can follow the same format if you want the change reflected in the release notes.

- **Scope**: Keep PRs focused. Run `just test` and `just fmt` (and fix any Clippy/ESLint issues) before requesting review.

## Rebranding and config

App name, publisher, docs URL, and GitHub repo are controlled by **`brand.json`** at the project root. After editing it, run `npm run sync-brand` so Tauri config and capabilities stay in sync. See [Rebranding](README.md#rebranding) in the README.

## Questions or issues

Open an issue on GitHub for bugs, feature ideas, or documentation gaps.
