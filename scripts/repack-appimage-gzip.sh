#!/usr/bin/env bash
# Repack an AppImage so its squashfs uses gzip compression.
# This allows AppImageLauncher (and older libsquashfs that only support xz/zlib) to read the image.
# Usage: repack-appimage-gzip.sh <path-to.AppImage>
# Requires: squashfs-tools (mksquashfs, unsquashfs)

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

# Type 2 AppImage: payload offset at byte 8, 8 bytes little-endian
OFFSET=$(od -An -t u8 -j 8 -N 8 -v "$APPIMAGE" | tr -d ' ')
if [ -z "$OFFSET" ] || [ "$OFFSET" -le 0 ]; then
  echo "Could not read payload offset from $APPIMAGE (not Type 2?)" >&2
  exit 1
fi

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

# Split runtime and payload
head -c "$OFFSET" "$APPIMAGE" > "$WORKDIR/runtime"
tail -c +$((OFFSET + 1)) "$APPIMAGE" > "$WORKDIR/payload.sqfs"

# Extract payload (supports zstd from current build)
unsquashfs -f -d "$WORKDIR/appdir" "$WORKDIR/payload.sqfs"

# Repack with gzip for AppImageLauncher compatibility
mksquashfs "$WORKDIR/appdir" "$WORKDIR/new.sqfs" -comp gzip -noappend

# Replace original with runtime + gzip squashfs
cat "$WORKDIR/runtime" "$WORKDIR/new.sqfs" > "$WORKDIR/new.AppImage"
chmod +x "$WORKDIR/new.AppImage"
mv "$WORKDIR/new.AppImage" "$APPIMAGE"

echo "Repacked $APPIMAGE with gzip compression"
