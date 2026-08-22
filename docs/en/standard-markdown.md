# Core portable Markdown

These forms are the recommended portable foundation in SilkMark and generally work in other modern Markdown viewers as well.

## Headings

```md
# H1
## H2
### H3
#### H4
##### H5
###### H6
```

Setext H1/H2:

```md
H1 heading
==========

H2 heading
----------
```

## Paragraphs and line breaks

```md
First paragraph.

Second paragraph.
```

A normal newline inside a paragraph is a soft break. Two trailing spaces create a hard break:

```md
First line.  
Second line.
```

## Emphasis

```md
*italic*
**bold**
**bold with *italic***
```

## Inline code

```md
Run `cargo build --release` to build.
```

Multiple backticks can delimit content that itself contains a backtick:

````md
``one ` character``
````

## Code blocks

````md
```rust
fn main() {
    println!("Hello");
}
```
````

Tilde fences are also supported:

```md
~~~text
Plain text block
~~~
```


### Indented code block

The CommonMark four-column indented code block is also supported:

```md
    fn main() {
        println!("Hello");
    }
```

For new documents, fenced code blocks are still recommended because they are clearer and can carry a language identifier.

## Blockquotes

```md
> A quote.
>
> A second paragraph.
```

Nested:

```md
> Outer
>> Inner
```

## Unordered lists

```md
- apple
- pear
- plum
```

`*` and `+` markers are also accepted.

## Ordered lists

```md
1. first
2. second
3. third
```

The CommonMark `)` ordered-list marker is also supported:

```md
1) first
2) second
```

An explicit starting number is preserved:

```md
7. seventh
8. eighth
```

## Nested blocks

```md
- top-level item
  - second level
    1. third level
```

A list item may also contain blockquotes and fenced code blocks.

## Thematic break

```md
---
```

Forms such as `***`, `___`, and `- - -` are also supported.

## Links

```md
[Documentation](guide.md)
[Rust](https://www.rust-lang.org/)
[Rust](https://www.rust-lang.org/ "Rust language")
[Installation](#installation)
[API / Types](api.md#types)
```

## Reference links

```md
The [Rust][rust] language.

[rust]: https://www.rust-lang.org/ "Rust"
```

Shortcut forms:

```md
[Documentation][]
[Documentation]: docs/index.md

[Home]
[Home]: index.md
```

## Images

```md
![Logo](images/logo.png)
![Remote image](https://example.org/image.png)
```

Reference image:

```md
![Logo][logo]

[logo]: images/logo.png "Project logo"
```

## Autolinks

URI autolink:

```md
<https://example.org/>
```

Email autolink:

```md
<user@example.com>
```

## Escapes

Use a backslash when Markdown punctuation should be literal:

```md
\*not italic\*
\[not a link\]
```

## HTML entities

```md
&amp;
&lt;
&gt;
&#169;
&#x1F642;
```

## Compatibility note

This chapter documents the portable forms that SilkMark supports reliably. SilkMark aims for practical CommonMark/GFM compatibility rather than special-casing every artificial or rare parser edge case in the specifications.

## CommonMark 1.0 release notes

SilkMark supports both `*`/`**` and `_`/`__` emphasis forms, CommonMark ASCII
backslash escapes, URI/e-mail autolinks, indented code, and fenced code with the
standard maximum of three leading spaces. Named character references use the
complete semicolon-terminated HTML5 name set required by CommonMark parsing.
Raw HTML remains intentionally inert and is displayed as text.
