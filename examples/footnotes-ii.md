# Footnotes II

A footnote can be referenced once.[^simple]

The same note can be referenced more than once: first here[^shared], and then here again.[^shared]

A complex footnote can contain several indented lines.[^complex]

[^simple]: A short footnote with an [HTTPS link](https://www.rust-lang.org/).

[^shared]: This note has multiple references. Use the numbered back-links below it to return.

[^complex]: First paragraph with **bold**, *italic*, and `inline code`.
    Second continuation line with a relative [document link](reader.md).

    - A list item inside the footnote
    - Another list item

    ```rust
    fn footnote_example() {
        println!("code inside a footnote");
    }
    ```
