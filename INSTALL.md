# SilkMark install

## Native build

Required packages:

```sh
apt install gcc libgtk-4-dev libcurl4-gnutls-dev
```

```sh
cargo build --release
```

The release binary is `target/release/silkmark`.

## .deb package (host toolchain)

```sh
sudo apt install cargo rustc pkg-config dpkg-dev libgtk-4-dev libgdk-pixbuf-2.0-dev libcurl4-openssl-dev
./tools/build-deb.sh
sudo dpkg -i silkmark.deb
```

`tools/build-deb.sh` builds the release binary, stages the package, resolves the
exact runtime dependencies with `dpkg-shlibdeps`, and creates `silkmark.deb`.
An optional first argument selects another output filename.

## .deb installer (reproducible container)

To build the `.deb` without depending on the host machine's installed libraries,
use the Podman-based build environment:

```sh
./tools/build-installer.sh
sudo dpkg -i installers/silkmark_*.deb
```

`tools/build-installer.sh` builds a Debian + Rust container image from
`tools/Containerfile`, runs `tools/build-deb.sh` inside it, and writes the
versioned installer to `installers/`. The container build is clean (a tmpfs
shadows `target/`), so the result is reproducible across machines with the same
architecture.

## Windows installer (MSYS2/GTK4)

The Windows installer is built from the MSYS2 MINGW64 subsystem, which provides
the GTK4 / libcurl / gdk-pixbuf development stack that `build.rs` resolves via
`pkg-config`. Install MSYS2, then in a MINGW64 shell:

```sh
pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-curl \
          mingw-w64-x86_64-gdk-pixbuf2 mingw-w64-x86_64-rust \
          mingw-w64-x86_64-nsis mingw-w64-x86_64-pkgconf
./tools/build-windows-installer.sh
```

This builds the release binary, stages a relocatable GTK application tree
(`bin/` + DLL closure, `lib/` gdk-pixbuf loaders, `share/` schemas and icons),
regenerates the gdk-pixbuf loaders cache and glib schemas for the bundle, and
runs NSIS to produce `installers/silkmark_<version>_win64-setup.exe`.

On GitHub Actions the same pipeline runs on `windows-latest` via
`.github/workflows/build-windows.yml` (CI) and the `release-windows` job of
`release.yml` (release assets).

## Prebuilt installer from releases

Prebuilt `.deb` and Windows `.exe` installers are published as GitHub Release
assets (no zip) on version tags. Download the latest from the GitHub Releases
page:

https://github.com/hg2ecz/silkmark/releases

See `CHANGELOG.md` for released versions.
