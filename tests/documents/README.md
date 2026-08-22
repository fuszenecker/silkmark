# Robustness corpus

These documents are intentionally small smoke-test inputs for manual rendering and parser regression work.

- `malformed.md`: truncated and structurally inconsistent Markdown that must not crash SilkMark.
- `unicode.md`: multilingual text, Unicode headings and links.
- `diagrams.md`: Mermaid and DOT flowchart smoke cases.

The automated large-document regression is generated in `src/md/tests.rs` so the repository does not need to carry a multi-megabyte fixture.
