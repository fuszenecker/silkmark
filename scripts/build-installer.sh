#!/bin/sh
set -eu

# Build a SilkMark .deb installer inside a reproducible Debian + Rust podman
# image, then place it in ./installers/.
#
# This reuses tools/build-deb.sh for the cargo release build and the
# dpkg-shlibdeps dependency resolution, but runs it in an isolated container so
# the resulting .deb does not depend on the host machine's installed libraries.
#
# Usage:  scripts/build-installer.sh
# Output: installers/silkmark_<version>_<arch>.deb

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

if ! command -v podman >/dev/null 2>&1; then
    echo "error: podman not found. Install Podman first." >&2
    exit 1
fi

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [ -z "$VERSION" ]; then
    echo "error: cannot read package version from Cargo.toml" >&2
    exit 1
fi

ARCH=$(dpkg --print-architecture 2>/dev/null || true)
if [ -z "$ARCH" ]; then
    case "$(uname -m)" in
        x86_64)  ARCH=amd64 ;;
        aarch64) ARCH=arm64 ;;
        *)       ARCH=$(uname -m) ;;
    esac
fi

IMAGE=silkmark-build
INSTALLERS="$ROOT/installers"
mkdir -p "$INSTALLERS"

# Build (or reuse) the Rust compiling environment image.
echo "==> Building podman image: $IMAGE"
podman build -t "$IMAGE" -f "$ROOT/scripts/Containerfile" "$ROOT"

# Run the package build inside the container. A tmpfs shadows the host's
# target/ so the build is clean and does not reuse host-compiled artifacts.
# Rootless podman maps container root to the current host user, so the .deb
# written to the bind-mounted installers/ directory is host-owned.
echo "==> Building .deb inside container"
podman run --rm \
    -v "$ROOT:/work" \
    --tmpfs /work/target \
    --workdir /work \
    "$IMAGE" \
    sh -c 'tools/build-deb.sh /work/installers/silkmark.deb'

OUT="$INSTALLERS/silkmark_${VERSION}_${ARCH}.deb"
mv -f "$INSTALLERS/silkmark.deb" "$OUT"

printf '\n==> Built installer: %s\n' "$OUT"
printf '==> Install with: sudo dpkg -i %s\n' "$OUT"
