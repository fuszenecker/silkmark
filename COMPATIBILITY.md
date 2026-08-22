# SilkMark Markdown compatibility profile

SilkMark targets a deliberately practical document-viewing profile rather than every Markdown dialect.

## Primary target

- CommonMark-style block and inline Markdown
  - four-column indented code blocks
  - ordered list markers ending in `.` or `)`
  - URI and email autolinks
- GitHub Flavored Markdown features useful for documentation
- GitHub-style mathematical delimiters, rendered by SilkMark's lightweight built-in math subset

## Supported document features

- ATX H1-H6 and Setext H1/H2 headings
- canonical, shareable heading fragments
- paragraphs, soft and hard line breaks
- emphasis, strong emphasis, strikethrough and inline code
- blockquotes and nested list/quote combinations
- unordered, ordered and task lists
- fenced code blocks using backticks or tildes
- language-labelled code blocks and lightweight syntax highlighting
- inline, reference and relative links
- inline and reference images (PNG, JPEG and GIF)
- pipe tables with GFM alignment markers
- autolinks
- footnotes, including multiline footnotes
- thematic breaks
- HTML entities
- inline and display mathematics

## Math syntax

SilkMark accepts these GitHub-style forms:

- `$...$`
- `$`...``$` safe inline form
- `$$...$$`
- fenced `math` blocks

The math command set is intentionally a subset of LaTeX. SilkMark does not embed TeX, MathJax, KaTeX or a WebView.

## Deliberate non-goals

- executing raw HTML
- CSS or JavaScript
- arbitrary TeX macros (`\\newcommand`, package loading, TikZ, etc.)
- browser-style HTML fallback
- silently interpreting unknown URL schemes

Raw HTML is displayed safely rather than executed. This is an intentional exception to full CommonMark rendering compatibility.

## Regression policy

Features listed above should have parser regression coverage before SilkMark 1.0. The visual smoke-test document is `examples/regression-suite.md`.

## Native diagrams

SilkMark includes documentation-oriented native rendering for a Mermaid flowchart subset and a Graphviz/DOT graph subset. These are deliberately not full Mermaid or Graphviz implementations. Unsupported diagram syntax remains visible as a highlighted fenced code block.
