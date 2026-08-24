## 1.0.6+0

- ship a CA bundle (`ca-bundle.crt`) next to the Windows `silkmark.exe` and point libcurl at it via `CURLOPT_CAINFO`, fixing `CURLE_SSL_CACERT_BADFILE` ("Problem with the SSL CA cert (path? access rights?)") on clean Windows installs where MSYS2 libcurl's compiled-in CA path does not exist
- keep HTTPS certificate verification intact; on Linux the bundled file is absent next to the binary, so libcurl keeps using the system trust store unchanged

## 1.0.5+1

- add explicit `--allow-http` opt-in for trusted LAN/intranet Markdown servers
- keep HTTPS-only networking as the default
- allow HTTP relative links, images, redirects and non-standard ports only while the opt-in is enabled
- apply `--allow-host` to both HTTP and HTTPS hosts
- recognize plain and angle-bracket HTTP autolinks in Markdown
- keep ftp:, data:, javascript: and remote file authorities rejected
- add `tools/build-installer.sh` and `tools/Containerfile`: build the `.deb` inside a reproducible Debian + Rust Podman image and output a versioned installer to `installers/`
- add GitHub Actions: `build` (cargo build + test), `build-installer` (`.deb` artifact), and `release` (publish `.deb` as a directly-downloadable Release asset on `v*` tags)
- add Windows installer pipeline: `installer/nsis/silkmark.nsi` (NSIS installer with `bin/lib/share` GTK layout, `.md` association, uninstaller), `tools/build-windows-installer.sh` (MSYS2/MINGW64 build, recursive DLL-closure staging, relocatable gdk-pixbuf loaders cache and glib schemas, icon themes), and `build-windows.yml` CI + `release-windows` release job publishing the `.exe` as a Release asset

## 1.0.4

- Fix duplicate `OnceLock` import in `src/net.rs` introduced by the 1.0.3 hardening pass.
- No runtime behavior changes.

## 1.0.3

- Remove an obsolete Markdown helper that caused a release-build dead-code warning.
- Harden internal state/cache text reads with strict byte limits before UTF-8 parsing.
- Create atomic state/cache temporary files with mode 0600 on Unix.
- Initialize libcurl globally once via `OnceLock`, avoiding repeated concurrent global initialization from worker threads.
- Add `SECURITY.md` documenting the executable-content boundary, resource limits, persistence hardening, and FFI review guidance.

## 1.0.2

- Fix a compile regression in Setext-heading preprocessing: restore the trimmed current-line binding used by heading detection.
- No intended parser behavior change beyond restoring the existing Setext logic.

## 1.0.1

- Fix generated CommonMark named-entity table: `&apos;` is now emitted as a valid Rust string literal.
- No functional Markdown or GTK behavior changes from 1.0.0.

## 1.0.0
- Stabilized GTK document replacement by clearing focus before removing a focused document subtree.
- Added underscore emphasis/strong handling with delimiter-side checks.
- Completed CommonMark ASCII backslash-escape punctuation handling.
- Added the full semicolon-terminated HTML5 named character-reference set used by CommonMark (2125 names), while raw HTML remains inert text.
- Enforced the CommonMark 0-3 leading-space rule for fenced code blocks.
- Added Debian/Ubuntu `tools/build-deb.sh` package builder using `dpkg-shlibdeps` and `dpkg-deb`.
- Removed temporary GTK trace instrumentation from the release source.
