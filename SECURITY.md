# SilkMark security notes

SilkMark is a native Markdown documentation browser. Its security model deliberately keeps rendered documents non-executable.

## Security boundaries

- Network documents and images use HTTPS only.
- Redirects are restricted to HTTPS and are re-checked against the optional host allowlist.
- `javascript:`, `data:`, plain HTTP, FTP, and unknown URI schemes are rejected.
- Raw HTML is rendered inertly; HTML, CSS, and JavaScript are not executed.
- Document, image, cache, session, bookmark, diagram, and graph inputs have explicit resource limits.
- Local `file://` URLs only resolve to local filesystem paths; remote file authorities are rejected.
- State and cache writes use atomic replacement. On Unix, newly written state/cache files are mode 0600.
- Persisted text metadata is read with hard byte limits before UTF-8 parsing.
- libcurl global initialization is performed once before use by worker threads.

## Native FFI

SilkMark intentionally calls GTK4, GLib, Cairo, GdkPixbuf, and libcurl through direct C FFI to avoid Rust package dependencies. This makes FFI pointer lifetime and callback ABI correctness a primary review area. Keep FFI declarations synchronized with the system library headers and avoid broad lifecycle refactors without runtime regression testing.

## Release gate

Before release, run:

```sh
./tools/release-check.sh
```

This checks formatting, runs Clippy with warnings denied, executes tests, and produces a release build.
