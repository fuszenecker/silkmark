# Proposals for reading book-length documents

Source: two real transcribed books used as test input, parsed with the unmodified
`src/md*.rs` in a standalone harness (no GTK4 development files were available, so
nothing below was measured in the GUI):

- Patrick Olivelle, *The Early Upaniṣads* — 1.3 MB, 13 126 lines, 153 headings, 206 table lines
- Richard Rutt, *Zhouyi* — 975 KB, 12 219 lines, 649 headings, 376 quote lines, 717 table lines

Parsing is not a problem: both documents parse in 40–60 ms (release build).

## Done on this branch (parser correctness, allowed by the feature freeze)

1. **Backslash hard line break.** A line ending in an unescaped `\` followed by another
   line of the same paragraph is now a hard break (CommonMark). Before, 358 lines in the
   Olivelle book (title page, verses) showed a literal `\` and ran together on one line.
2. **Nested lists four or more columns deep.** `    - item` directly after a list item
   was taken for an indented code block, so every third list level (45 blocks in
   Olivelle's per-Upaniṣad contents) rendered as a monospaced `- ...` blob. It now stays
   a nested list item. Two previously red tests (`nested_list_indent_is_kept`,
   `nested_lists_keep_increasing_indent`) pass again.

## Proposed, not implemented

Ordered by expected value for long books.

1. **Render large documents incrementally.** `render_tab` clones the whole `Vec<Line>`,
   clones every line again in the loop, and creates one `GtkLabel` per line in a single
   synchronous pass (about 12 000 widgets for these books). Append in chunks of a few
   hundred lines from a `g_idle_add` callback and run the anchor/scroll jump after the
   last chunk. Needs measuring first: `--stats` already prints render time.
2. **Sidebar contents for books.** The Contents list stops after 200 headings and
   truncates titles at 34 characters. Zhouyi has 649 headings, so the contents ends
   roughly a third of the way into the book. Options: collapsible H2 sections (open only
   the current one), a filter entry, full title as tooltip.
3. **Diacritic-insensitive search.** Search lowercases only (`render.rs`), so `Upanisad`
   does not find `Upaniṣad`, and `Brhadaranyaka` does not find `Bṛhadāraṇyaka`. Fold
   combining marks and the common Latin-extended letters (ā ī ū ṛ ṣ ṇ ṭ ś ṃ ḥ ñ ü) on both
   sides, with a toggle or a smart-case rule. This matters for Sanskrit and pinyin texts.
4. **Section navigation keys.** `]` / `[` for next/previous heading and a one-line
   "current section" indicator, so a reader can orient themselves in a 13 000-line page.
   Reader zoom (80–160 %) already exists; a maximum text column width would be the
   companion setting for wide windows.
5. **Footnotes in plain-text books.** Olivelle marks notes with superscripts (¹ ²) and
   keeps the text under `### Notes` at the end of a section. A heuristic that links
   superscript digits to the matching entry would be useful, but it is guesswork on
   unstructured text; consider only as an opt-in.
6. **Test suite.** Under the unmodified repo layout `src/md/tests.rs` refers to
   `../examples/regression-suite.md` (resolves to `src/examples/`, which does not exist)
   and calls the private `inline::parse_inline`. I could not run `cargo test` here
   (GTK4 development files missing), but in my harness both stopped compilation until
   I shimmed them. After that, 6 of 95 tests still fail (ordered lists over nine digits,
   GFM table alignment, task + strikethrough, quoted lists, spaced thematic breaks, plus
   lists); the same 6 fail without my changes. See the existing note about the red suite.
