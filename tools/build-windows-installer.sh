#!/usr/bin/env bash
# Build the SilkMark Windows installer (.exe) inside an MSYS2 MINGW64 shell.
#
# Prerequisites (MSYS2 packages, MINGW64 subsystem):
#   mingw-w64-x86_64-gtk4 mingw-w64-x86_64-curl mingw-w64-x86_64-gdk-pixbuf2
#   mingw-w64-x86_64-rust mingw-w64-x86_64-nsis mingw-w64-x86_64-pkgconf
#
# What it does:
#   1. cargo build --release           (Rust links the MSYS2 GTK4/libcurl/gdk-pixbuf)
#   2. stage a relocatable GTK app tree under target/windows-stage/{bin,lib,share}
#   3. collect the full DLL closure of silkmark.exe into bin/
#   4. regenerate the gdk-pixbuf loaders cache and glib schemas for the bundle
#   5. makensis -> installers/silkmark_<version>_win64-setup.exe
#
# Output: installers/silkmark_<version>_win64-setup.exe
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)
if [ -z "${VERSION:-}" ]; then
    echo "error: cannot read package version from Cargo.toml" >&2
    exit 1
fi
echo "==> Version: $VERSION"

# Sanity: make sure we really are in a MINGW64 environment with the libraries.
if ! pkg-config --exists gtk4 libcurl gdk-pixbuf-2.0; then
    echo "error: pkg-config cannot find gtk4/libcurl/gdk-pixbuf-2.0." >&2
    echo "       Install the mingw-w64-x86_64-gtk4 / curl / gdk-pixbuf2 MSYS2 packages." >&2
    exit 1
fi

MKEW_PREFIX=$(cygpath -u "$(pkg-config --variable=prefix gtk4)")
echo "==> GTK prefix: $MKEW_PREFIX"
if [ ! -d "$MKEW_PREFIX/bin" ]; then
    echo "error: expected a bin/ directory under the GTK prefix ($MKEW_PREFIX)" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
echo "==> cargo build --release"
cargo build --release

EXE="target/release/silkmark.exe"
if [ ! -f "$EXE" ]; then
    echo "error: build did not produce $EXE" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Stage a relocatable GTK application tree
# ---------------------------------------------------------------------------
STAGE="$ROOT/target/windows-stage"
echo "==> Staging into: $STAGE"
rm -rf "$STAGE"
mkdir -p "$STAGE/bin"

# The binary itself
cp "$EXE" "$STAGE/bin/silkmark.exe"

# ---------------------------------------------------------------------------
# Collect the full DLL closure of silkmark.exe (transitive, mingw-only).
# ldd prints "  name => /mingw64/bin/x.dll (addr)" lines; we keep only paths
# under the mingw prefix and recurse until the set stops growing. System DLLs
# (C:/Windows/SYSTEM32) are deliberately left out.
# ---------------------------------------------------------------------------
collect_dll_closure() {
    local root_exe="$1"
    local seen_file
    seen_file="$(mktemp)"
    : > "$seen_file"
    local queue=("$root_exe")
    while [ "${#queue[@]}" -gt 0 ]; do
        local cur="${queue[0]}"
        queue=("${queue[@]:1}")
        local ldd_out
        ldd_out="$(ldd "$cur" 2>/dev/null || true)"
        while IFS= read -r line; do
            # Parse: "  libfoo.dll => /mingw64/bin/libfoo.dll (0x...)"
            local p
            p=$(printf '%s\n' "$line" | sed -n 's/.*=> \([^ ]*\) .*/\1/p')
            [ -z "$p" ] && continue
            case "$p" in
                "$MKEW_PREFIX"/*) ;;  # keep only the mingw prefix DLLs
                *) continue ;;
            esac
            [ -f "$p" ] || continue
            if grep -Fxq "$p" "$seen_file"; then
                continue
            fi
            echo "$p" >> "$seen_file"
            queue+=("$p")
        done <<<"$ldd_out"
    done
    cat "$seen_file"
    rm -f "$seen_file"
}

echo "==> Collecting DLL closure"
mapfile -t DLLS < <(collect_dll_closure "$EXE")
if [ "${#DLLS[@]}" -lt 5 ]; then
    echo "warning: only ${#DLLS[@]} DLLs collected; closure walk may have failed" >&2
fi
for dll in "${DLLS[@]}"; do
    cp -f "$dll" "$STAGE/bin/"
done
echo "    staged ${#DLLS[@]} DLLs"

# ---------------------------------------------------------------------------
# gdk-pixbuf loaders: copy the loader modules and regenerate a relocatable
# loaders.cache by placing gdk-pixbuf-query-loaders.exe in stage/bin (it
# computes libdir as <bindir>/../lib, matching the staged layout).
# ---------------------------------------------------------------------------
LOADERS_SRC=$(ls -d "$MKEW_PREFIX"/lib/gdk-pixbuf-2.0/*/loaders 2>/dev/null | head -n1 || true)
if [ -n "$LOADERS_SRC" ] && [ -d "$LOADERS_SRC" ]; then
    LOADERS_REL="${LOADERS_SRC#"$MKEW_PREFIX"/}"
    LOADERS_STAGE="$STAGE/lib/$LOADERS_REL"
    mkdir -p "$LOADERS_STAGE"
    cp -f "$LOADERS_SRC"/*.dll "$LOADERS_STAGE/" 2>/dev/null || true

    QL="$MKEW_PREFIX/bin/gdk-pixbuf-query-loaders.exe"
    if [ -f "$QL" ]; then
        cp -f "$QL" "$STAGE/bin/"
        echo "==> Regenerating gdk-pixbuf loaders.cache (relocatable)"
        (cd "$STAGE/bin" && ./gdk-pixbuf-query-loaders.exe --update-cache || \
            echo "warning: loaders.cache regeneration failed; using prebuilt cache" >&2)
        # The cache is written into the loaders dir; drop the tool afterwards.
        rm -f "$STAGE/bin/gdk-pixbuf-query-loaders.exe"
    else
        echo "warning: gdk-pixbuf-query-loaders.exe not found; skipping cache" >&2
    fi
else
    echo "warning: gdk-pixbuf loaders directory not found" >&2
fi

# ---------------------------------------------------------------------------
# glib schemas: ship a compiled gschemas.compiled (relocatable).
# ---------------------------------------------------------------------------
SCHEMAS_SRC="$MKEW_PREFIX/share/glib-2.0/schemas"
if [ -d "$SCHEMAS_SRC" ]; then
    SCHEMAS_STAGE="$STAGE/share/glib-2.0/schemas"
    mkdir -p "$SCHEMAS_STAGE"
    cp -f "$SCHEMAS_SRC"/*.gschema.xml "$SCHEMAS_STAGE/" 2>/dev/null || true
    if command -v glib-compile-schemas >/dev/null 2>&1; then
        echo "==> Compiling glib schemas"
        glib-compile-schemas "$SCHEMAS_STAGE" || \
            echo "warning: glib-compile-schemas failed" >&2
    else
        echo "warning: glib-compile-schemas not found; copying any prebuilt compiled schemas" >&2
        cp -f "$SCHEMAS_SRC/gschemas.compiled" "$SCHEMAS_STAGE/" 2>/dev/null || true
    fi
else
    echo "warning: glib schemas directory not found" >&2
fi

# ---------------------------------------------------------------------------
# Icon themes (Adwaita is the GTK default; hicolor carries the index).
# ---------------------------------------------------------------------------
mkdir -p "$STAGE/share/icons"
for theme in Adwaita hicolor; do
    if [ -d "$MKEW_PREFIX/share/icons/$theme" ]; then
        echo "==> Copying icon theme: $theme"
        cp -a "$MKEW_PREFIX/share/icons/$theme" "$STAGE/share/icons/"
    fi
done

# ---------------------------------------------------------------------------
# Run NSIS. makensis is a native Windows program, so it needs Windows paths.
# ---------------------------------------------------------------------------
NSI="$ROOT/installer/nsis/silkmark.nsi"
if ! command -v makensis >/dev/null 2>&1; then
    echo "error: makensis not found. Install mingw-w64-x86_64-nsis." >&2
    exit 1
fi

INSTALLERS="$ROOT/installers"
mkdir -p "$INSTALLERS"
OUT="$INSTALLERS/silkmark_${VERSION}_win64-setup.exe"

STAGE_WIN=$(cygpath -w "$STAGE")
OUT_WIN=$(cygpath -w "$OUT")
NSI_WIN=$(cygpath -w "$NSI")

echo "==> Running NSIS"
makensis -V2 \
    "-DAPP_VERSION=$VERSION" \
    "-DSTAGE_DIR=$STAGE_WIN" \
    "-DOUT_FILE=$OUT_WIN" \
    "$NSI_WIN"

if [ ! -f "$OUT" ]; then
    echo "error: installer was not produced at $OUT" >&2
    exit 1
fi

printf '\n==> Built installer: %s\n' "$OUT"
printf '==> Install with: %s\n' "$OUT_WIN"
