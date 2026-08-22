# SilkMark pre-1.0 stabilization

SilkMark 0.40 is a feature-freeze checkpoint before the 1.0 preparation cycle.
The goal is predictable behavior, warning-free builds, and small, auditable FFI boundaries rather than new Markdown syntax.

## Release gate

A release candidate should pass all of the following on the current stable Rust toolchain. The same gate is available as `tools/release-check.sh`:

```bash
rustc --version
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

It should also be smoke-tested with:

- a local Markdown file with relative links and images;
- an HTTPS Markdown document;
- back/forward/reload and multiple tabs;
- bookmarks and session restore;
- search and deep-link fragments;
- static and animated images;
- tables, footnotes, fenced code, syntax highlighting, and math;
- `--offline`, `--disk-cache`, `--allow-host`, and `--stats`.

## Feature freeze

Until the 1.0 compatibility contract is finalized, changes after v0.40 should normally be limited to:

1. compiler/clippy fixes;
2. parser correctness and regression fixes;
3. FFI lifetime/safety fixes;
4. performance regressions;
5. documentation and packaging corrections.

New syntax or UI features should be deferred unless they are required to resolve a compatibility defect.

## Unsafe/FFI rule

Direct GTK, GDK-Pixbuf, Cairo, GLib, and libcurl calls remain inside explicit unsafe boundary functions.
Parser, Markdown model, persistence, URL policy, and other pure logic should stay safe Rust.
Any new raw-pointer lifetime assumption should be local, obvious from the surrounding callback/adapter, and documented when it is not self-evident.

## Robustness audit

Before release, also review `SAFETY.md` and smoke-test the documents in `tests/documents/`, especially malformed input and diagram fallback behavior.


## State-file hardening (v0.46)

Persistent state is treated as untrusted input. Session restore clamps window dimensions, active-tab selection, scroll positions, and the number of restored tabs. Bookmark restore accepts only the resource schemes SilkMark itself can bookmark (`https://` and `file://`) and caps the number and line size of entries.

## Diagram resource limits (v0.46)

Native Mermaid and Graphviz/DOT rendering intentionally has conservative limits on source size, node count, edge count, and label length. When a diagram exceeds those limits, rendering must fail safely and the fenced source remains available through the normal highlighted-code fallback. This protects the GTK widget tree and Cairo layout from pathological documentation input.
