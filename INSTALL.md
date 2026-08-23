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

## Prebuilt installer from releases

Prebuilt `.deb` installers are published as GitHub Release assets (no zip) on
version tags. Download the latest from the GitHub Releases page, or directly:

```sh
curl -LO https://github.com/<owner>/silkmark/releases/latest/download/silkmark.deb
```

See `CHANGELOG.md` for released versions.
