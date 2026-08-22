#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

for tool in cargo dpkg dpkg-deb dpkg-shlibdeps install sed; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: required tool not found: $tool" >&2
        echo "Debian/Ubuntu build tools: sudo apt install cargo rustc pkg-config dpkg-dev libgtk-4-dev libgdk-pixbuf-2.0-dev libcurl4-openssl-dev" >&2
        exit 1
    fi
done

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [ -z "$VERSION" ]; then
    echo "error: cannot read package version from Cargo.toml" >&2
    exit 1
fi
ARCH=$(dpkg --print-architecture)
OUT=${1:-"$ROOT/silkmark.deb"}
WORK="$ROOT/target/debian-package"
STAGE="$WORK/silkmark_${VERSION}_${ARCH}"

rm -rf "$WORK"
mkdir -p "$STAGE/DEBIAN" \
    "$STAGE/usr/bin" \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/doc/silkmark"

cargo build --release
install -m 0755 target/release/silkmark "$STAGE/usr/bin/silkmark"
install -m 0644 packaging/silkmark.desktop "$STAGE/usr/share/applications/silkmark.desktop"
install -m 0644 README.md "$STAGE/usr/share/doc/silkmark/README.md"
install -m 0644 LICENSE "$STAGE/usr/share/doc/silkmark/copyright"
install -m 0644 CHANGELOG.md "$STAGE/usr/share/doc/silkmark/changelog"

# Ask Debian's own shlibs database for the dependencies of the binary. This
# automatically follows distribution transitions such as libcurl4 -> libcurl4t64.
# dpkg-shlibdeps expects minimal Debian source metadata, so provide an isolated
# temporary control file instead of modifying the project root.
mkdir -p "$WORK/debian"
cat > "$WORK/debian/control" <<META
Source: silkmark
Section: doc
Priority: optional
Maintainer: Zsolt Krüpl
Standards-Version: 4.6.0

Package: silkmark
Architecture: any
Description: SilkMark package staging metadata
META
DEPENDS=$(cd "$WORK" && dpkg-shlibdeps -O -e"$STAGE/usr/bin/silkmark" 2>/dev/null \
    | sed -n 's/^shlibs:Depends=//p')
if [ -z "$DEPENDS" ]; then
    echo "error: dpkg-shlibdeps could not determine runtime dependencies" >&2
    exit 1
fi

cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: silkmark
Version: $VERSION
Section: doc
Priority: optional
Architecture: $ARCH
Depends: $DEPENDS
Maintainer: Zsolt Krüpl
Description: Fast lightweight native Markdown document browser
 SilkMark is a GTK4 Markdown documentation browser with syntax highlighting,
 tables, footnotes, native math rendering, Mermaid flowcharts and a Graphviz
 DOT subset. Raw HTML, CSS and JavaScript are not executed.
CONTROL

# Installed files are root-owned even when the script is run as a normal user.
dpkg-deb --root-owner-group --build "$STAGE" "$OUT"
printf 'Built: %s\n' "$OUT"
printf 'Install: sudo dpkg -i %s\n' "$OUT"
