## Unreleased

- treat an unescaped trailing backslash as a CommonMark hard line break
- keep list items indented four or more columns as nested list items instead of indented code blocks
- virtualize the document view: a `GtkListView` creates widgets only for visible blocks instead of one label per line (book-length documents open in tens of milliseconds and use about half the memory)
- fetch and parse documents on the worker thread; the UI thread no longer parses
- find no longer re-renders the document: hits are counted from a precomputed text index and only visible rows are rebuilt
- same-document fragment links just scroll; section jumps and restored positions now scroll reliably in long documents
- store the session reading position as block index plus fraction; older pixel offsets are ignored
- sidebar: left-aligned tree with indentation, one-line ellipsized titles (full text in the tooltip), highlight of the current section, no horizontal scrollbar, up to 1000 headings (was 200 and truncated at 34 characters)
- sidebar no longer jumps to its top when it is rebuilt or when a contents entry is clicked
- tab titles use the document's first heading, else the decoded file name (no more `%20%E2%80%94`); bookmark titles are decoded too
- the focus leaves the address bar once a page has loaded, so Page Down and the arrow keys scroll the page; Space/Shift+Space page, j/k scroll by a line (the key handler now runs in the capture phase)
- add `docs/proposals-book-reading.md` with proposals for book-length documents

## 1.0.5

- add explicit `--allow-http` opt-in for trusted LAN/intranet Markdown servers
- keep HTTPS-only networking as the default
- allow HTTP relative links, images, redirects and non-standard ports only while the opt-in is enabled
- apply `--allow-host` to both HTTP and HTTPS hosts
- recognize plain and angle-bracket HTTP autolinks in Markdown
- keep ftp:, data:, javascript: and remote file authorities rejected

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
