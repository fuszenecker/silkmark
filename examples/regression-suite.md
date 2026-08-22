# SilkMark regression suite

This document is a visual smoke test for the supported Markdown profile.

Setext heading
==============

## Inline formatting

Plain text with *italic*, **bold**, ~~deleted~~, and ``code with a ` backtick``.

Escaped punctuation: \*literal stars\* and \[literal brackets\].

## Links

Inline: [relative document](local/sub/child.md "Relative title")

Reference: [Rust][rust]

Autolink: <https://www.rust-lang.org/>

[rust]: https://www.rust-lang.org/ "Rust"

## Lists and quotes

- outer
  - inner
    7. ordered child

> - quoted list
>   - nested quoted list

- code below

  ```rust
  fn main() {
      println!("SilkMark");
  }
  ```

## Tasks

- [x] supported
- [ ] unchecked

## Table

| Left | Center | Right |
| :--- | :----: | ----: |
| *a* | `b` | [c](https://example.org/) |
| long wrapping cell content for layout testing | middle | 42 |

## Footnotes

First reference.[^shared] Second reference.[^shared]

[^shared]: Multiline footnote with **formatting**.
    Continued line and [link](https://example.org/).

## Math

Inline: $E=mc^2$.

$$
\lim_{x\to0}\frac{\sin x}{x}=1
$$

```math
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
```

## Rule

- - -

## Raw HTML safety

<script>alert("must not execute")</script>
