# Flatpak packaging for Announcemint

The Flatpak manifest and metainfo are **generated from `brand.json`** by `npm run sync-brand` (run automatically before dev/build). Edit `brand.json` and run `npm run sync-brand` to update.

## Optional brand.json fields for Flatpak

- `developerName` – default: "Nicholas Santiago"
- `projectLicense` – default: "GPL-3.0-or-later"

## Prerequisites

```bash
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
```

## Building locally

```bash
just flatpak
```

Or manually:

1. Build the .deb package (on Linux):
   ```bash
   npm run tauri build -- --bundles deb --ci
   ```

2. Prepare the Flatpak build:
   ```bash
   npm run prepare-flatpak-deb
   ```

3. Build the Flatpak:
   ```bash
   flatpak-builder --repo=repo --force-clean build-dir flatpak/manifest.yml
   flatpak build-bundle repo $(cat flatpak/.app-id).flatpak $(cat flatpak/.app-id)
   ```

4. Install the bundle:
   ```bash
   flatpak install *.flatpak
   ```

## Running

```bash
flatpak run $(cat flatpak/.app-id)
```

## CI

The Flatpak is built automatically in CI on pull requests and when the workflow is manually triggered with "Build installers" enabled. The `.flatpak` bundle is uploaded as the `linux-flatpak-installer` artifact.
