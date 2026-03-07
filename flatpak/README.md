# Flatpak packaging for Announcemint

This directory contains the Flatpak manifest and metadata for building Announcemint as a Flatpak package.

## Prerequisites

```bash
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
```

## Building locally

1. Build the .deb package (on Linux):
   ```bash
   npm run tauri build -- --bundles deb --ci
   ```

2. Copy the deb into this directory:
   ```bash
   cp src-tauri/target/release/bundle/deb/*.deb flatpak/announcemint.deb
   ```

3. Build the Flatpak:
   ```bash
   flatpak-builder --repo=repo --force-clean build-dir flatpak/com.santiagon610.Announcemint.yml
   flatpak build-bundle repo announcemint.flatpak com.santiagon610.Announcemint
   ```

4. Install the bundle:
   ```bash
   flatpak install announcemint.flatpak
   ```

## Running

```bash
flatpak run com.santiagon610.Announcemint
```

## CI

The Flatpak is built automatically in CI on pull requests and when the workflow is manually triggered with "Build installers" enabled. The `.flatpak` bundle is uploaded as the `linux-flatpak-installer` artifact.
