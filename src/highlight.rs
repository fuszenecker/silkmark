//! Tiny dependency-free syntax highlighter for fenced Markdown code blocks.
//! It intentionally aims for readable documentation, not full compiler-grade parsing.

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn span(class: &str, s: &str) -> String {
    let body = esc(s);
    match class {
        "kw" => format!("<b><span foreground=\"#7c3aed\">{body}</span></b>"),
        "str" => format!("<span foreground=\"#15803d\">{body}</span>"),
        "com" => format!("<i><span foreground=\"#64748b\">{body}</span></i>"),
        "num" => format!("<span foreground=\"#b45309\">{body}</span>"),
        "key" => format!("<span foreground=\"#0369a1\">{body}</span>"),
        "lit" => format!("<b><span foreground=\"#be123c\">{body}</span></b>"),
        _ => body,
    }
}

pub fn supported(language: &str) -> bool {
    matches!(
        norm(language).as_str(),
        "rust"
            | "c"
            | "cpp"
            | "csharp"
            | "java"
            | "kotlin"
            | "go"
            | "swift"
            | "nim"
            | "javascript"
            | "typescript"
            | "sql"
            | "lua"
            | "ruby"
            | "php"
            | "dart"
            | "scala"
            | "zig"
            | "python"
            | "shell"
            | "json"
            | "toml"
            | "yaml"
            | "html"
            | "latex"
            | "markdown"
            | "asciidoc"
            | "rst"
            | "css"
            | "json5"
            | "ini"
            | "dockerfile"
            | "mermaid"
            | "dot"
    )
}

pub fn render(language: &str, text: &str) -> String {
    match norm(language).as_str() {
        "rust" => c_like(text, RUST_KW, true),
        "c" => c_like(text, C_KW, false),
        "cpp" => c_like(text, CPP_KW, false),
        "csharp" => c_like(text, CSHARP_KW, false),
        "java" => c_like(text, JAVA_KW, false),
        "kotlin" => c_like(text, KOTLIN_KW, false),
        "go" => c_like(text, GO_KW, false),
        "swift" => c_like(text, SWIFT_KW, false),
        "javascript" => c_like(text, JAVASCRIPT_KW, false),
        "typescript" => c_like(text, TYPESCRIPT_KW, false),
        "php" => c_like(text, PHP_KW, false),
        "dart" => c_like(text, DART_KW, false),
        "scala" => c_like(text, SCALA_KW, false),
        "zig" => c_like(text, ZIG_KW, false),
        "nim" => line_lang(text, NIM_KW, b'#'),
        "lua" => lua(text),
        "ruby" => line_lang(text, RUBY_KW, b'#'),
        "sql" => sql(text),
        "python" => python(text),
        "shell" => shell(text),
        "json" => json_like(text, true),
        "toml" => toml(text),
        "yaml" => yaml(text),
        "html" => html(text),
        "latex" => latex(text),
        "markdown" => markdown(text),
        "asciidoc" => asciidoc(text),
        "rst" => rst(text),
        "css" => css(text),
        "json5" => json5(text),
        "ini" => ini(text),
        "dockerfile" => dockerfile(text),
        "mermaid" => mermaid(text),
        "dot" => dot(text),
        _ => esc(text),
    }
}

fn norm(s: &str) -> String {
    match s.trim().to_ascii_lowercase().as_str() {
        "rs" => "rust".into(),
        "c++" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".into(),
        "cs" | "c#" | "dotnet" => "csharp".into(),
        "kt" | "kts" => "kotlin".into(),
        "golang" => "go".into(),
        "js" | "jsx" | "node" | "nodejs" => "javascript".into(),
        "ts" | "tsx" => "typescript".into(),
        "rb" => "ruby".into(),
        "php8" => "php".into(),
        "py" => "python".into(),
        "sh" | "bash" | "zsh" | "fish" => "shell".into(),
        "yml" => "yaml".into(),
        "htm" | "xhtml" | "xml" | "svg" => "html".into(),
        "tex" | "latex2e" => "latex".into(),
        "md" | "mdown" => "markdown".into(),
        "adoc" | "asc" => "asciidoc".into(),
        "rest" | "restructuredtext" => "rst".into(),
        "jsonc" => "json5".into(),
        "cfg" | "conf" | "properties" => "ini".into(),
        "docker" | "containerfile" => "dockerfile".into(),
        "mmd" => "mermaid".into(),
        "graphviz" | "gv" => "dot".into(),
        x => x.into(),
    }
}

const RUST_KW: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
];
const C_KW: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for",
    "goto", "if", "inline", "int", "long", "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
    "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "_Bool",
];
const CPP_KW: &[&str] = &[
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "constexpr",
    "continue",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "if",
    "inline",
    "int",
    "long",
    "namespace",
    "new",
    "noexcept",
    "nullptr",
    "operator",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "template",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
];
const CSHARP_KW: &[&str] = &[
    "abstract",
    "as",
    "base",
    "bool",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "checked",
    "class",
    "const",
    "continue",
    "decimal",
    "default",
    "delegate",
    "do",
    "double",
    "else",
    "enum",
    "event",
    "explicit",
    "extern",
    "false",
    "finally",
    "fixed",
    "float",
    "for",
    "foreach",
    "if",
    "implicit",
    "in",
    "int",
    "interface",
    "internal",
    "is",
    "lock",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "operator",
    "out",
    "override",
    "params",
    "private",
    "protected",
    "public",
    "readonly",
    "record",
    "ref",
    "return",
    "sbyte",
    "sealed",
    "short",
    "sizeof",
    "stackalloc",
    "static",
    "string",
    "struct",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "uint",
    "ulong",
    "unchecked",
    "unsafe",
    "ushort",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
    "async",
    "await",
    "var",
    "yield",
];
const JAVA_KW: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "try",
    "void",
    "volatile",
    "while",
    "true",
    "false",
    "null",
    "record",
    "sealed",
    "permits",
    "var",
    "yield",
];
const KOTLIN_KW: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
    "by",
    "catch",
    "constructor",
    "delegate",
    "dynamic",
    "field",
    "file",
    "finally",
    "get",
    "import",
    "init",
    "param",
    "property",
    "receiver",
    "set",
    "setparam",
    "where",
    "actual",
    "abstract",
    "annotation",
    "companion",
    "const",
    "crossinline",
    "data",
    "enum",
    "expect",
    "external",
    "final",
    "infix",
    "inline",
    "inner",
    "internal",
    "lateinit",
    "noinline",
    "open",
    "operator",
    "out",
    "override",
    "private",
    "protected",
    "public",
    "reified",
    "sealed",
    "suspend",
    "tailrec",
    "vararg",
];
const GO_KW: &[&str] = &[
    "break",
    "default",
    "func",
    "interface",
    "select",
    "case",
    "defer",
    "go",
    "map",
    "struct",
    "chan",
    "else",
    "goto",
    "package",
    "switch",
    "const",
    "fallthrough",
    "if",
    "range",
    "type",
    "continue",
    "for",
    "import",
    "return",
    "var",
    "true",
    "false",
    "nil",
    "iota",
];
const SWIFT_KW: &[&str] = &[
    "associatedtype",
    "class",
    "deinit",
    "enum",
    "extension",
    "fileprivate",
    "func",
    "import",
    "init",
    "inout",
    "internal",
    "let",
    "open",
    "operator",
    "private",
    "precedencegroup",
    "protocol",
    "public",
    "rethrows",
    "static",
    "struct",
    "subscript",
    "typealias",
    "var",
    "break",
    "case",
    "continue",
    "default",
    "defer",
    "do",
    "else",
    "fallthrough",
    "for",
    "guard",
    "if",
    "in",
    "repeat",
    "return",
    "switch",
    "where",
    "while",
    "as",
    "Any",
    "catch",
    "false",
    "is",
    "nil",
    "super",
    "self",
    "Self",
    "throw",
    "throws",
    "true",
    "try",
];
const NIM_KW: &[&str] = &[
    "addr",
    "and",
    "as",
    "asm",
    "bind",
    "block",
    "break",
    "case",
    "cast",
    "concept",
    "const",
    "continue",
    "converter",
    "defer",
    "discard",
    "distinct",
    "div",
    "do",
    "elif",
    "else",
    "end",
    "enum",
    "except",
    "export",
    "finally",
    "for",
    "from",
    "func",
    "generic",
    "if",
    "import",
    "in",
    "include",
    "interface",
    "is",
    "isnot",
    "iterator",
    "let",
    "macro",
    "method",
    "mixin",
    "mod",
    "nil",
    "not",
    "notin",
    "object",
    "of",
    "or",
    "out",
    "proc",
    "ptr",
    "raise",
    "ref",
    "return",
    "shl",
    "shr",
    "static",
    "template",
    "try",
    "tuple",
    "type",
    "using",
    "var",
    "when",
    "while",
    "with",
    "without",
    "xor",
    "yield",
];
const JAVASCRIPT_KW: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "null",
    "of",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "async",
];
const TYPESCRIPT_KW: &[&str] = &[
    "abstract",
    "any",
    "as",
    "asserts",
    "async",
    "await",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "constructor",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "of",
    "override",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "satisfies",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
];
const LUA_KW: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in", "local", "nil", "not", "or",
    "repeat", "return", "then", "true", "until", "while",
];
const RUBY_KW: &[&str] = &[
    "BEGIN", "END", "alias", "and", "begin", "break", "case", "class", "def", "defined?", "do", "else", "elsif", "end", "ensure",
    "false", "for", "if", "in", "module", "next", "nil", "not", "or", "redo", "rescue", "retry", "return", "self", "super",
    "then", "true", "undef", "unless", "until", "when", "while", "yield",
];
const PHP_KW: &[&str] = &[
    "abstract",
    "and",
    "array",
    "as",
    "break",
    "callable",
    "case",
    "catch",
    "class",
    "clone",
    "const",
    "continue",
    "declare",
    "default",
    "do",
    "echo",
    "else",
    "elseif",
    "empty",
    "enddeclare",
    "endfor",
    "endforeach",
    "endif",
    "endswitch",
    "endwhile",
    "enum",
    "eval",
    "exit",
    "extends",
    "final",
    "finally",
    "fn",
    "for",
    "foreach",
    "function",
    "global",
    "goto",
    "if",
    "implements",
    "include",
    "include_once",
    "instanceof",
    "insteadof",
    "interface",
    "isset",
    "list",
    "match",
    "namespace",
    "new",
    "or",
    "print",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "require_once",
    "return",
    "static",
    "switch",
    "throw",
    "trait",
    "try",
    "unset",
    "use",
    "var",
    "while",
    "xor",
    "yield",
    "true",
    "false",
    "null",
];
const DART_KW: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "base",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "Function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "null",
    "of",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "sealed",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typedef",
    "var",
    "void",
    "when",
    "while",
    "with",
    "yield",
];
const SCALA_KW: &[&str] = &[
    "abstract",
    "case",
    "catch",
    "class",
    "def",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "for",
    "forSome",
    "given",
    "if",
    "implicit",
    "import",
    "lazy",
    "match",
    "new",
    "null",
    "object",
    "opaque",
    "override",
    "package",
    "private",
    "protected",
    "return",
    "sealed",
    "super",
    "then",
    "this",
    "throw",
    "trait",
    "transparent",
    "true",
    "try",
    "type",
    "using",
    "val",
    "var",
    "while",
    "with",
    "yield",
];
const ZIG_KW: &[&str] = &[
    "addrspace",
    "align",
    "allowzero",
    "and",
    "anyframe",
    "anytype",
    "asm",
    "async",
    "await",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fn",
    "for",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "packed",
    "pub",
    "resume",
    "return",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "try",
    "union",
    "unreachable",
    "usingnamespace",
    "var",
    "volatile",
    "while",
    "true",
    "false",
    "null",
    "undefined",
];
const SQL_KW: &[&str] = &[
    "add",
    "all",
    "alter",
    "and",
    "as",
    "asc",
    "between",
    "by",
    "case",
    "check",
    "column",
    "constraint",
    "create",
    "database",
    "default",
    "delete",
    "desc",
    "distinct",
    "drop",
    "else",
    "end",
    "exists",
    "foreign",
    "from",
    "full",
    "group",
    "having",
    "in",
    "index",
    "inner",
    "insert",
    "into",
    "is",
    "join",
    "key",
    "left",
    "like",
    "limit",
    "not",
    "null",
    "on",
    "or",
    "order",
    "outer",
    "primary",
    "references",
    "right",
    "select",
    "set",
    "table",
    "then",
    "union",
    "unique",
    "update",
    "values",
    "view",
    "when",
    "where",
    "with",
    "true",
    "false",
];

const PY_KW: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except", "False",
    "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
    "return", "True", "try", "while", "with", "yield",
];
const SH_KW: &[&str] = &[
    "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in", "select", "then", "time", "until", "while",
];

fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}
fn is_num_start(c: u8) -> bool {
    c.is_ascii_digit()
}
fn push_plain_char(out: &mut String, text: &str, i: &mut usize) {
    let Some(ch) = text[*i..].chars().next() else {
        return;
    };
    out.push_str(&esc(&ch.to_string()));
    *i += ch.len_utf8();
}

fn c_like(text: &str, kws: &[&str], rust: bool) -> String {
    let b = text.as_bytes();
    let mut o = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            o.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            let j = text[i + 2..].find("*/").map(|n| i + 2 + n + 2).unwrap_or(b.len());
            o.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == q {
                    j += 1;
                    break;
                }
                j += 1;
            }
            o.push_str(&span("str", &text[i..j]));
            i = j;
            continue;
        }
        if rust && i + 1 < b.len() && b[i] == b'r' && b[i + 1] == b'#' { /* leave raw strings readable */ }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1
            }
            let w = &text[i..j];
            if kws.contains(&w) {
                o.push_str(&span("kw", w))
            } else if matches!(w, "true" | "false" | "NULL" | "nullptr") {
                o.push_str(&span("lit", w))
            } else {
                o.push_str(&esc(w))
            }
            i = j;
            continue;
        }
        if is_num_start(b[i]) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'.' | b'_' | b'x' | b'X')) {
                j += 1
            }
            o.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut o, text, &mut i);
    }
    o
}

fn python(text: &str) -> String {
    line_lang(text, PY_KW, b'#')
}
fn shell(text: &str) -> String {
    line_lang(text, SH_KW, b'#')
}
fn line_lang(text: &str, kws: &[&str], comment: u8) -> String {
    let b = text.as_bytes();
    let mut o = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == comment {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            o.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == q {
                    j += 1;
                    break;
                }
                j += 1
            }
            o.push_str(&span("str", &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1
            }
            let w = &text[i..j];
            if kws.contains(&w) {
                o.push_str(&span("kw", w))
            } else if matches!(w, "True" | "False" | "None") {
                o.push_str(&span("lit", w))
            } else {
                o.push_str(&esc(w))
            }
            i = j;
            continue;
        }
        if is_num_start(b[i]) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'.') {
                j += 1
            }
            o.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut o, text, &mut i);
    }
    o
}

fn lua(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'-' && b[i + 1] == b'-' {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == q {
                    j += 1;
                    break;
                }
                j += 1;
            }
            out.push_str(&span("str", &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1;
            }
            let word = &text[i..j];
            if LUA_KW.contains(&word) {
                out.push_str(&span("kw", word));
            } else {
                out.push_str(&esc(word));
            }
            i = j;
            continue;
        }
        if is_num_start(b[i]) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'.' | b'_')) {
                j += 1;
            }
            out.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn sql(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'-' && b[i + 1] == b'-' {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            let j = text[i + 2..].find("*/").map(|n| i + 2 + n + 2).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'\'' || b[i] == b'"' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == q {
                    if j + 1 < b.len() && b[j + 1] == q {
                        j += 2;
                        continue;
                    }
                    j += 1;
                    break;
                }
                j += 1;
            }
            out.push_str(&span("str", &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1;
            }
            let word = &text[i..j];
            let lower = word.to_ascii_lowercase();
            if SQL_KW.contains(&lower.as_str()) {
                out.push_str(&span(if matches!(lower.as_str(), "true" | "false" | "null") { "lit" } else { "kw" }, word));
            } else {
                out.push_str(&esc(word));
            }
            i = j;
            continue;
        }
        if is_num_start(b[i]) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_digit() || matches!(b[j], b'.' | b'e' | b'E' | b'+' | b'-')) {
                j += 1;
            }
            out.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn json_like(text: &str, _strict: bool) -> String {
    let b = text.as_bytes();
    let mut o = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1
            }
            let mut k = j;
            while k < b.len() && b[k].is_ascii_whitespace() {
                k += 1
            }
            o.push_str(&span(if k < b.len() && b[k] == b':' { "key" } else { "str" }, &text[i..j]));
            i = j;
            continue;
        }
        if b[i].is_ascii_digit() || b[i] == b'-' {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_digit() || matches!(b[j], b'.' | b'e' | b'E' | b'+' | b'-')) {
                j += 1
            }
            o.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1
            }
            let w = &text[i..j];
            o.push_str(&span(if matches!(w, "true" | "false" | "null") { "lit" } else { "" }, w));
            i = j;
            continue;
        }
        push_plain_char(&mut o, text, &mut i);
    }
    o
}
fn toml(text: &str) -> String {
    let mut o = String::new();
    for (n, line) in text.split_inclusive('\n').enumerate() {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        o.push_str(&esc(lead));
        if t.starts_with('#') {
            o.push_str(&span("com", t))
        } else if t.starts_with('[') {
            o.push_str(&span("key", t))
        } else if let Some(eq) = t.find('=') {
            o.push_str(&span("key", t[..eq].trim_end()));
            o.push_str(&esc(&t[t[..eq].trim_end().len()..=eq]));
            o.push_str(&value_tokens(&t[eq + 1..]))
        } else {
            o.push_str(&esc(t))
        }
        if nl {
            o.push('\n')
        }
        let _ = n;
    }
    o
}
fn yaml(text: &str) -> String {
    let mut o = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        o.push_str(&esc(lead));
        if t.starts_with('#') {
            o.push_str(&span("com", t))
        } else if let Some(col) = t.find(':') {
            if !t[..col].contains(' ') {
                o.push_str(&span("key", &t[..col]));
                o.push_str(":");
                o.push_str(&value_tokens(&t[col + 1..]));
            } else {
                o.push_str(&esc(t))
            }
        } else {
            o.push_str(&value_tokens(t))
        }
        if nl {
            o.push('\n')
        }
    }
    o
}
fn value_tokens(s: &str) -> String {
    let t = s.trim_start();
    let lead = &s[..s.len() - t.len()];
    let mut o = esc(lead);
    if t.starts_with('"') || t.starts_with('\'') {
        o.push_str(&span("str", t))
    } else if matches!(t, "true" | "false" | "null" | "True" | "False" | "None") {
        o.push_str(&span("lit", t))
    } else if t.parse::<f64>().is_ok() {
        o.push_str(&span("num", t))
    } else {
        o.push_str(&esc(t))
    }
    o
}

fn html(text: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with("<!--") {
            let end = text[i + 4..].find("-->").map(|n| i + 4 + n + 3).unwrap_or(text.len());
            out.push_str(&span("com", &text[i..end]));
            i = end;
            continue;
        }
        if text.as_bytes()[i] == b'<' {
            let end = text[i..].find('>').map(|n| i + n + 1).unwrap_or(text.len());
            out.push_str(&highlight_tag(&text[i..end]));
            i = end;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn highlight_tag(tag: &str) -> String {
    let bytes = tag.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != q {
                j += 1;
            }
            if j < bytes.len() {
                j += 1;
            }
            out.push_str(&span("str", &tag[i..j]));
            i = j;
        } else if bytes[i].is_ascii_alphabetic() || bytes[i] == b'!' || bytes[i] == b'?' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || matches!(bytes[j], b'-' | b':' | b'_' | b'!' | b'?')) {
                j += 1;
            }
            out.push_str(&span(if i <= 2 { "kw" } else { "key" }, &tag[i..j]));
            i = j;
        } else {
            push_plain_char(&mut out, tag, &mut i);
        }
    }
    out
}

fn latex(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'\\' {
            let mut j = i + 1;
            if j < b.len() && b[j].is_ascii_alphabetic() {
                while j < b.len() && (b[j].is_ascii_alphabetic() || b[j] == b'@') {
                    j += 1;
                }
            } else if j < b.len() {
                j += 1;
            }
            out.push_str(&span("kw", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'{' || b[i] == b'}' || b[i] == b'[' || b[i] == b']' || b[i] == b'$' {
            out.push_str(&span("key", &text[i..i + 1]));
            i += 1;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn markdown(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        out.push_str(&esc(lead));
        if t.starts_with("<!--") {
            out.push_str(&span("com", t));
        } else if t.starts_with('#') {
            out.push_str(&span("kw", t));
        } else if t.starts_with("> ") || t == ">" {
            out.push_str(&span("com", t));
        } else if t.starts_with("```") || t.starts_with("~~~") {
            out.push_str(&span("key", t));
        } else if t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ") {
            out.push_str(&span("key", &t[..2]));
            out.push_str(&esc(&t[2..]));
        } else {
            out.push_str(&esc(t));
        }
        if nl {
            out.push('\n');
        }
    }
    out
}

fn asciidoc(text: &str) -> String {
    line_document(text, &["=", "==", "===", "===="], "//")
}

fn rst(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        if t.starts_with(".. ") {
            out.push_str(&span("com", core));
        } else if t.starts_with(":") && t.contains(':') {
            out.push_str(&span("key", core));
        } else if !t.is_empty() && t.chars().all(|c| matches!(c, '=' | '-' | '~' | '^' | '"' | '#')) {
            out.push_str(&span("kw", core));
        } else {
            out.push_str(&esc(core));
        }
        if nl {
            out.push('\n');
        }
    }
    out
}

fn css(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            let j = text[i + 2..].find("*/").map(|n| i + 2 + n + 2).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == q {
                    j += 1;
                    break;
                }
                j += 1;
            }
            out.push_str(&span("str", &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) || b[i] == b'-' {
            let mut j = i + 1;
            while j < b.len() && (is_word(b[j]) || b[j] == b'-') {
                j += 1
            }
            let mut k = j;
            while k < b.len() && b[k].is_ascii_whitespace() {
                k += 1
            }
            out.push_str(&span(if k < b.len() && b[k] == b':' { "key" } else { "" }, &text[i..j]));
            i = j;
            continue;
        }
        if is_num_start(b[i]) {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'.' | b'%' | b'-')) {
                j += 1
            }
            out.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn json5(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            let j = text[i..].find('\n').map(|n| i + n).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            let j = text[i + 2..].find("*/").map(|n| i + 2 + n + 2).unwrap_or(b.len());
            out.push_str(&span("com", &text[i..j]));
            i = j;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j = (j + 2).min(b.len());
                    continue;
                }
                if b[j] == q {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let mut k = j;
            while k < b.len() && b[k].is_ascii_whitespace() {
                k += 1
            }
            out.push_str(&span(if k < b.len() && b[k] == b':' { "key" } else { "str" }, &text[i..j]));
            i = j;
            continue;
        }
        if is_word(b[i]) {
            let mut j = i + 1;
            while j < b.len() && is_word(b[j]) {
                j += 1
            }
            let w = &text[i..j];
            out.push_str(&span(if matches!(w, "true" | "false" | "null" | "Infinity" | "NaN") { "lit" } else { "" }, w));
            i = j;
            continue;
        }
        if is_num_start(b[i]) || b[i] == b'-' || b[i] == b'+' {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || matches!(b[j], b'.' | b'_' | b'+' | b'-')) {
                j += 1
            }
            out.push_str(&span("num", &text[i..j]));
            i = j;
            continue;
        }
        push_plain_char(&mut out, text, &mut i);
    }
    out
}

fn ini(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        out.push_str(&esc(lead));
        if t.starts_with(';') || t.starts_with('#') {
            out.push_str(&span("com", t))
        } else if t.starts_with('[') && t.ends_with(']') {
            out.push_str(&span("kw", t))
        } else if let Some(eq) = t.find('=') {
            out.push_str(&span("key", t[..eq].trim_end()));
            out.push_str(&esc(&t[t[..eq].trim_end().len()..=eq]));
            out.push_str(&value_tokens(&t[eq + 1..]));
        } else {
            out.push_str(&esc(t))
        }
        if nl {
            out.push('\n')
        }
    }
    out
}

fn dockerfile(text: &str) -> String {
    const KW: &[&str] = &[
        "ADD",
        "ARG",
        "CMD",
        "COPY",
        "ENTRYPOINT",
        "ENV",
        "EXPOSE",
        "FROM",
        "HEALTHCHECK",
        "LABEL",
        "MAINTAINER",
        "ONBUILD",
        "RUN",
        "SHELL",
        "STOPSIGNAL",
        "USER",
        "VOLUME",
        "WORKDIR",
    ];
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        out.push_str(&esc(lead));
        if t.starts_with('#') {
            out.push_str(&span("com", t))
        } else {
            let end = t.find(char::is_whitespace).unwrap_or(t.len());
            let word = &t[..end];
            if KW.contains(&word.to_ascii_uppercase().as_str()) {
                out.push_str(&span("kw", word));
                out.push_str(&esc(&t[end..]));
            } else {
                out.push_str(&esc(t));
            }
        }
        if nl {
            out.push('\n')
        }
    }
    out
}

fn mermaid(text: &str) -> String {
    const KW: &[&str] = &[
        "flowchart",
        "graph",
        "sequenceDiagram",
        "classDiagram",
        "stateDiagram",
        "stateDiagram-v2",
        "erDiagram",
        "journey",
        "gantt",
        "pie",
        "mindmap",
        "timeline",
        "subgraph",
        "end",
        "participant",
        "actor",
        "Note",
        "loop",
        "alt",
        "else",
        "opt",
        "par",
        "and",
        "rect",
        "activate",
        "deactivate",
    ];
    line_document_words(text, KW, "%%")
}

fn dot(text: &str) -> String {
    const KW: &[&str] = &["strict", "graph", "digraph", "subgraph", "node", "edge"];
    c_like(text, KW, false)
}

fn line_document(text: &str, headings: &[&str], comment: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        if t.starts_with(comment) {
            out.push_str(&span("com", core))
        } else if headings.iter().any(|h| t == *h || t.starts_with(&format!("{h} "))) {
            out.push_str(&span("kw", core))
        } else {
            out.push_str(&esc(core))
        }
        if nl {
            out.push('\n')
        }
    }
    out
}

fn line_document_words(text: &str, kws: &[&str], comment: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let nl = line.ends_with('\n');
        let core = line.trim_end_matches('\n');
        let t = core.trim_start();
        let lead = &core[..core.len() - t.len()];
        out.push_str(&esc(lead));
        if t.starts_with(comment) {
            out.push_str(&span("com", t))
        } else {
            let end = t.find(char::is_whitespace).unwrap_or(t.len());
            let w = &t[..end];
            if kws.contains(&w) {
                out.push_str(&span("kw", w));
                out.push_str(&esc(&t[end..]));
            } else {
                out.push_str(&esc(t));
            }
        }
        if nl {
            out.push('\n')
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases() {
        for lang in [
            "rs", "c++", "c#", "cs", "kt", "golang", "js", "tsx", "rb", "bash", "yml", "html", "xml", "tex", "md", "adoc", "rst",
            "css", "jsonc", "ini", "docker", "mmd", "dot",
        ] {
            assert!(supported(lang), "missing alias: {lang}");
        }
    }
    #[test]
    fn rust_keywords_and_strings() {
        let h = render("rust", "fn main(){ let s=\"hi\"; }");
        assert!(h.contains("<b>"));
        assert!(h.contains("hi"));
    }
    #[test]
    fn json_keys() {
        let h = render("json", "{\"name\": true}");
        assert!(h.contains("0369a1"));
        assert!(h.contains("be123c"));
    }
    #[test]
    fn added_language_keywords() {
        assert!(render("java", "public class Demo {}").contains("7c3aed"));
        assert!(render("nim", "proc main() = discard").contains("7c3aed"));
        assert!(render("sql", "SELECT * FROM users WHERE id = 42").contains("7c3aed"));
        assert!(render("html", "<section class=\"doc\">text</section>").contains("7c3aed"));
        assert!(render("tex", "\\section{Intro} % note").contains("7c3aed"));
        assert!(render("docker", "FROM rust:latest").contains("7c3aed"));
    }
}
