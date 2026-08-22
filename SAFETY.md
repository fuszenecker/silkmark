# SilkMark safety and robustness notes

SilkMark intentionally uses direct C ABI bindings for GTK4, GLib/GObject, Cairo, GdkPixbuf and libcurl. The crate therefore has an explicit FFI boundary instead of hiding `unsafe` behind third-party Rust bindings.

## FFI rules

- Callback `data` / `userdata` pointers are checked for null before dereference.
- Heap callback contexts created with `Box::into_raw` have exactly one registered destroy callback that reconstructs them with `Box::from_raw`.
- GObject references retained beyond a local call must be balanced by `g_object_unref` during page/application cleanup.
- GTK widget access remains on the GTK main thread. Worker threads return owned Rust data through channels; they do not manipulate GTK objects directly.
- libcurl write/header callbacks validate multiplication overflow, null pointers and configured download limits before creating Rust slices.

## Persistent state

Session, bookmark and disk-cache writes use `storage::atomic_write`:

1. create a unique temporary file in the destination directory;
2. write all bytes;
3. `sync_all` the file;
4. rename it into place;
5. remove the temporary file on failure.

This prevents a normal interrupted write from leaving a partially written state file. Corrupt or unreadable state/cache data is treated as a cache/state miss rather than a fatal startup error.

## Parser robustness

Malformed Markdown, math and diagram input should degrade to literal text or highlighted source where possible. Parser code should not panic on untrusted document contents. Regression tests include malformed/truncated constructs and a large synthetic document.

## Release gate

Run before release:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Then manually smoke-test local files, HTTPS, images, session restore, search, Mermaid and DOT rendering.


## Persistent input and diagram limits

Session, bookmark, cache, Markdown, Mermaid, and DOT inputs are not trusted merely because they are local files. Restoration code must clamp counts and numeric values before creating GTK state. Native diagram parsers impose explicit source/node/edge/label limits; exceeding a limit is a normal parse failure and must fall back to source display rather than panic or partially render.
