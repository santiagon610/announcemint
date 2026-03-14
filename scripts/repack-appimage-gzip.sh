#!/usr/bin/env bash
# Repack an AppImage so its squashfs uses gzip compression.
# This allows AppImageLauncher (and older libsquashfs that only support xz/zlib) to read the image.
# Usage: repack-appimage-gzip.sh <path-to.AppImage>
# Requires: squashfs-tools (mksquashfs)
# Uses the AppImage runtime's --appimage-extract for reliable extraction across formats and compression.

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <path-to.AppImage>" >&2
  exit 1
fi

APPIMAGE="$1"
if [ ! -f "$APPIMAGE" ]; then
  echo "Not a file: $APPIMAGE" >&2
  exit 1
fi
APPIMAGE=$(cd "$(dirname "$APPIMAGE")" && pwd)/$(basename "$APPIMAGE")

# Ensure executable for --appimage-extract / --appimage-offset
chmod +x "$APPIMAGE"

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

# Copy AppImage to workdir so --appimage-extract works in a clean dir
cp "$APPIMAGE" "$WORKDIR/input.AppImage"
chmod +x "$WORKDIR/input.AppImage"
cd "$WORKDIR"

# Use AppImage runtime's built-in extraction (handles format variations, zstd, etc.)
if ! ./input.AppImage --appimage-extract >/dev/null 2>&1; then
  echo "AppImage does not support --appimage-extract (not Type 2?)" >&2
  exit 1
fi

# Get payload offset from runtime for reconstructing the file
OFFSET=$(./input.AppImage --appimage-offset 2>/dev/null || true)
if [ -z "$OFFSET" ] || [ "$OFFSET" -le 0 ]; then
  echo "Could not get payload offset from AppImage" >&2
  exit 1
fi

# Extract runtime (bytes before payload)
head -c "$OFFSET" input.AppImage > runtime

# Set WebKit to use CPU rendering so EGL is never used (avoids EGL_BAD_PARAMETER crash on some setups).
# Must be set before the app binary runs, so we patch AppRun here; setting it in Rust main() is too late.
# Only patch if AppRun is a script (starts with #!); do not modify binary AppRun.
if [ -f squashfs-root/AppRun ] && head -c 2 squashfs-root/AppRun | grep -q '^#!'; then
  (echo "$(head -n 1 squashfs-root/AppRun)"
   echo 'export WEBKIT_SKIA_ENABLE_CPU_RENDERING=1'
   tail -n +2 squashfs-root/AppRun) > squashfs-root/AppRun.new
  mv squashfs-root/AppRun.new squashfs-root/AppRun
  chmod +x squashfs-root/AppRun
fi

# Repack extracted squashfs-root with gzip for AppImageLauncher compatibility
mksquashfs squashfs-root new.sqfs -comp gzip -noappend

# Reconstruct AppImage
cat runtime new.sqfs > new.AppImage
chmod +x new.AppImage
mv new.AppImage "$APPIMAGE"

echo "Repacked $APPIMAGE with gzip compression"
