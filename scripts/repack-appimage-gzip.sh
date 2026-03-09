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

# Repack extracted squashfs-root with gzip for AppImageLauncher compatibility
mksquashfs squashfs-root new.sqfs -comp gzip -noappend

# Reconstruct AppImage
cat runtime new.sqfs > new.AppImage
chmod +x new.AppImage
mv new.AppImage "$APPIMAGE"

echo "Repacked $APPIMAGE with gzip compression"
