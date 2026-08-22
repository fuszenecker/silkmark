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
