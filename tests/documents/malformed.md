# Malformed-input smoke corpus

[unterminated link](https://example.org/foo_(bar

```rust
fn main() {
    println!("unterminated fence");

[^note]: Footnote with an unterminated nested fence.
    ```text
    still footnote

<https://example.org/no-closing-angle

&not-a-real-entity; &#x110000;

| uneven | table |
| --- |
| one | two | three |

> > - nested quote/list
continuation with deliberately odd indentation
