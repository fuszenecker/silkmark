use super::*;

pub(super) fn parse_image_with_refs(s: &str, refs: &ReferenceMap) -> Option<Link> {
    let t = s.trim();
    if !t.starts_with("![") {
        return None;
    }
    let rb = t[2..].find(']')? + 2;
    let label = &t[2..rb];
    let after = &t[rb + 1..];
    if after.starts_with('(') {
        let rp0 = find_balanced_paren(&after[1..])?;
        if !after[rp0 + 2..].trim().is_empty() {
            return None;
        }
        let inner = after[1..1 + rp0].trim();
        let (target, title) = markdown_destination(inner)?;
        return Some(Link { label: label.to_string(), target, title });
    }
    if after.starts_with('[') {
        let end = after[1..].find(']')?;
        if !after[end + 2..].trim().is_empty() {
            return None;
        }
        let raw_id = &after[1..1 + end];
        let id = if raw_id.is_empty() { label } else { raw_id };
        let (target, title) = refs.get(&normalize_reference_label(id))?;
        return Some(Link { label: label.to_string(), target: target.clone(), title: title.clone() });
    }
    if after.trim().is_empty() {
        let (target, title) = refs.get(&normalize_reference_label(label))?;
        return Some(Link { label: label.to_string(), target: target.clone(), title: title.clone() });
    }
    None
}

pub(super) fn find_balanced_paren(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut escaped = false;
    let mut angle = false;
    for (i, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '<' && depth == 0 {
            angle = true;
            continue;
        }
        if ch == '>' && angle {
            angle = false;
            continue;
        }
        if angle {
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Extract a Markdown link/image destination. Supports plain URLs,
/// `<URL with spaces>`, balanced parentheses and the common optional `"title"` suffix.
pub(super) fn markdown_destination(inner: &str) -> Option<(String, Option<String>)> {
    let s = inner.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix('<') {
        let end = rest.find('>')?;
        let url = &rest[..end];
        if url.is_empty() {
            return None;
        }
        let title = parse_optional_title(rest[end + 1..].trim());
        return Some((url.to_string(), title));
    }
    let mut end = s.len();
    for (i, ch) in s.char_indices() {
        if ch.is_whitespace() {
            end = i;
            break;
        }
    }
    let url = s[..end].trim();
    if url.is_empty() {
        return None;
    }
    let title = parse_optional_title(s[end..].trim());
    Some((url.to_string(), title))
}

pub(super) fn parse_optional_title(s: &str) -> Option<String> {
    let t = s.trim();
    if t.len() >= 2 {
        let q = t.as_bytes()[0];
        if (q == b'"' || q == b'\'') && t.as_bytes()[t.len() - 1] == q {
            return Some(t[1..t.len() - 1].to_string());
        }
    }
    None
}

pub(super) fn is_table_row(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && t.contains('|') && split_table_cells(t).len() >= 2
}

pub(super) fn is_table_separator(s: &str) -> bool {
    if !is_table_row(s) {
        return false;
    }
    s.trim_matches('|').split('|').all(|cell| {
        let x = cell.trim().trim_matches(':');
        !x.is_empty() && x.chars().all(|c| c == '-')
    })
}

pub(super) fn split_table_cells(s: &str) -> Vec<String> {
    let inner = s.trim().strip_prefix('|').unwrap_or(s.trim());
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            cur.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            cur.push(ch);
            escaped = true;
            continue;
        }
        if ch == '|' {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

pub(super) fn parse_table_cells(s: &str) -> Vec<String> {
    split_table_cells(s)
}

pub(super) fn parse_table_alignments(s: &str) -> Vec<TableAlign> {
    split_table_cells(s)
        .into_iter()
        .map(|cell| {
            let t = cell.trim();
            match (t.starts_with(':'), t.ends_with(':')) {
                (true, true) => TableAlign::Center,
                (false, true) => TableAlign::Right,
                _ => TableAlign::Left,
            }
        })
        .collect()
}

pub(super) fn parse_table_markup(s: &str, refs: &ReferenceMap) -> (String, Vec<Link>) {
    let cells = parse_table_cells(s);
    let mut out = String::new();
    let mut links = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            out.push_str("  |  ");
        }
        let (m, mut l) = parse_inline_ctx(cell, refs);
        out.push_str(&m);
        links.append(&mut l);
    }
    (out, links)
}

pub fn parse_inline_public(s: &str) -> (String, Vec<Link>) {
    parse_inline(s)
}

pub fn strip_inline_markup(s: &str) -> String {
    strip_inline_markup_inner(s)
}

fn parse_inline(s: &str) -> (String, Vec<Link>) {
    let refs = ReferenceMap::new();
    parse_inline_ctx(s, &refs)
}

pub(super) fn parse_inline_ctx(s: &str, refs: &ReferenceMap) -> (String, Vec<Link>) {
    let mut out = String::with_capacity(s.len() + 16);
    let mut links = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let tail = &s[i..];
        if tail.starts_with('\\') {
            if let Some(ch) = tail[1..].chars().next() {
                if ch.is_ascii_punctuation() {
                    match ch {
                        '&' => out.push_str("&amp;"),
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        _ => out.push(ch),
                    }
                    i += 1 + ch.len_utf8();
                    continue;
                }
            }
        }
        // GitHub-compatible Markdown-safe inline math: $`...`$
        if tail.starts_with("$`") {
            if let Some(end) = tail[2..].find("`$") {
                let expr = &tail[2..2 + end];
                out.push_str("<span font_family=\"serif\"><i>");
                out.push_str(&crate::math::render(expr));
                out.push_str("</i></span>");
                i += end + 4;
                continue;
            }
        }
        if tail.starts_with('$') && !tail.starts_with("$$") {
            if let Some(end) = tail[1..].find('$') {
                let expr = &tail[1..1 + end];
                out.push_str("<span font_family=\"serif\"><i>");
                out.push_str(&crate::math::render(expr));
                out.push_str("</i></span>");
                i += end + 2;
                continue;
            }
        }
        if tail.starts_with("~~") {
            if let Some(end) = tail[2..].find("~~") {
                out.push_str("<span strikethrough=\"true\">");
                let (inner, mut inner_links) = parse_inline_ctx(&tail[2..2 + end], refs);
                out.push_str(&inner);
                links.append(&mut inner_links);
                out.push_str("</span>");
                i += 4 + end;
                continue;
            }
        }
        if tail.starts_with("**") && emphasis_open_allowed(s, i, 2, '*') {
            if let Some(end) = find_emphasis_close(&tail[2..], '*', 2) {
                out.push_str("<b>");
                let (inner, mut inner_links) = parse_inline_ctx(&tail[2..2 + end], refs);
                out.push_str(&inner);
                links.append(&mut inner_links);
                out.push_str("</b>");
                i += 4 + end;
                continue;
            }
        }
        if tail.starts_with('*') && emphasis_open_allowed(s, i, 1, '*') {
            if let Some(end) = find_emphasis_close(&tail[1..], '*', 1) {
                out.push_str("<i>");
                let (inner, mut inner_links) = parse_inline_ctx(&tail[1..1 + end], refs);
                out.push_str(&inner);
                links.append(&mut inner_links);
                out.push_str("</i>");
                i += 2 + end;
                continue;
            }
        }
        if tail.starts_with("__") && emphasis_open_allowed(s, i, 2, '_') {
            if let Some(end) = find_emphasis_close(&tail[2..], '_', 2) {
                out.push_str("<b>");
                let (inner, mut inner_links) = parse_inline_ctx(&tail[2..2 + end], refs);
                out.push_str(&inner);
                links.append(&mut inner_links);
                out.push_str("</b>");
                i += 4 + end;
                continue;
            }
        }
        if tail.starts_with('_') && emphasis_open_allowed(s, i, 1, '_') {
            if let Some(end) = find_emphasis_close(&tail[1..], '_', 1) {
                out.push_str("<i>");
                let (inner, mut inner_links) = parse_inline_ctx(&tail[1..1 + end], refs);
                out.push_str(&inner);
                links.append(&mut inner_links);
                out.push_str("</i>");
                i += 2 + end;
                continue;
            }
        }
        if tail.starts_with('`') {
            let ticks = tail.as_bytes().iter().take_while(|&&b| b == b'`').count();
            let marker = "`".repeat(ticks);
            if let Some(end) = tail[ticks..].find(&marker) {
                let mut code = tail[ticks..ticks + end].replace('\n', " ");
                // CommonMark trims one surrounding space when both sides have it
                // and the span is not made only of spaces.
                if code.starts_with(' ') && code.ends_with(' ') && code.chars().any(|c| c != ' ') && code.len() >= 2 {
                    code.remove(0);
                    code.pop();
                }
                out.push_str("<tt>");
                out.push_str(&escape_markup(&code));
                out.push_str("</tt>");
                i += ticks + end + ticks;
                continue;
            }
        }
        if tail.starts_with("[^") {
            if let Some(rb) = tail[2..].find(']') {
                let id = &tail[2..2 + rb];
                if !id.is_empty() {
                    let frag = format!("#fn_{}", slugify(id));
                    out.push_str("<a href=\"");
                    out.push_str(&escape_attr(&frag));
                    out.push_str("\"><sup>");
                    out.push_str(&escape_markup(id));
                    out.push_str("</sup></a>");
                    links.push(Link { label: id.into(), target: frag, title: Some(format!("Footnote {}", id)) });
                    i += rb + 3;
                    continue;
                }
            }
        }
        if tail.starts_with('&') {
            if let Some((decoded, used)) = decode_html_entity(tail) {
                out.push_str(&escape_markup(&decoded));
                i += used;
                continue;
            }
        }
        if tail.starts_with("<https://") {
            if let Some(end) = tail.find('>') {
                let target = &tail[1..end];
                out.push_str("<a href=\"");
                out.push_str(&escape_attr(target));
                out.push_str("\">");
                out.push_str(&escape_markup(target));
                out.push_str("</a>");
                links.push(Link { label: target.into(), target: target.into(), title: None });
                i += end + 1;
                continue;
            }
        }
        if tail.starts_with('<') {
            if let Some(end) = tail.find('>') {
                let address = &tail[1..end];
                if is_email_autolink(address) {
                    let target = format!("mailto:{}", address);
                    out.push_str("<a href=\"");
                    out.push_str(&escape_attr(&target));
                    out.push_str("\">");
                    out.push_str(&escape_markup(address));
                    out.push_str("</a>");
                    links.push(Link { label: address.into(), target, title: None });
                    i += end + 1;
                    continue;
                }
            }
        }
        if tail.starts_with("https://") {
            let mut end = tail.len();
            for (j, ch) in tail.char_indices() {
                if ch.is_whitespace() || ch == '<' || ch == '>' {
                    end = j;
                    break;
                }
            }
            while end > 0 {
                let Some(ch) = tail[..end].chars().next_back() else { break };
                if matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}') {
                    end -= ch.len_utf8();
                } else {
                    break;
                }
            }
            if end > "https://".len() {
                let target = &tail[..end];
                out.push_str("<a href=\"");
                out.push_str(&escape_attr(target));
                out.push_str("\">");
                out.push_str(&escape_markup(target));
                out.push_str("</a>");
                links.push(Link { label: target.into(), target: target.into(), title: None });
                i += end;
                continue;
            }
        }
        if tail.starts_with('[') {
            if let Some(rb0) = tail[1..].find(']') {
                let rb = rb0 + 1;
                let label = &tail[1..rb];
                let after = &tail[rb + 1..];
                let mut resolved: Option<(String, Option<String>, usize)> = None;
                if after.starts_with('(') {
                    if let Some(rp0) = find_balanced_paren(&after[1..]) {
                        let inner = &after[1..1 + rp0];
                        if let Some((target, title)) = markdown_destination(inner) {
                            resolved = Some((target, title, rb + 1 + rp0 + 2));
                        }
                    }
                } else if after.starts_with('[') {
                    if let Some(end) = after[1..].find(']') {
                        let raw_id = &after[1..1 + end];
                        let id = if raw_id.is_empty() { label } else { raw_id };
                        if let Some((target, title)) = refs.get(&normalize_reference_label(id)) {
                            resolved = Some((target.clone(), title.clone(), rb + 1 + end + 2));
                        }
                    }
                } else if let Some((target, title)) = refs.get(&normalize_reference_label(label)) {
                    resolved = Some((target.clone(), title.clone(), rb + 1));
                }
                if let Some((target, title, used)) = resolved {
                    let (label_markup, _) = parse_inline_ctx(label, refs);
                    out.push_str("<a href=\"");
                    out.push_str(&escape_attr(&target));
                    out.push_str("\">");
                    out.push_str(&label_markup);
                    out.push_str("</a>");
                    // Nested links are not valid Markdown; link labels are rendered but nested targets are ignored.
                    links.push(Link { label: strip_inline_markup_inner(label), target, title });
                    i += used;
                    continue;
                }
            }
        }
        let Some(ch) = tail.chars().next() else { break };
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
        i += ch.len_utf8();
    }
    (out, links)
}

fn is_email_autolink(s: &str) -> bool {
    let Some((local, domain)) = s.rsplit_once('@') else { return false };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return false;
    }
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        return false;
    }
    if !local.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'.' | b'!'
                    | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'*'
                    | b'+'
                    | b'-'
                    | b'/'
                    | b'='
                    | b'?'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'{'
                    | b'|'
                    | b'}'
                    | b'~'
            )
    }) {
        return false;
    }
    domain.split('.').all(|part| {
        !part.is_empty()
            && !part.starts_with('-')
            && !part.ends_with('-')
            && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    })
}

pub(super) fn decode_html_entity(s: &str) -> Option<(String, usize)> {
    let semi = s.find(';')?;
    if semi > 64 {
        return None;
    }
    let name = &s[1..semi];
    let decoded = if name.starts_with("#x") || name.starts_with("#X") {
        char::from_u32(u32::from_str_radix(&name[2..], 16).ok()?)?.to_string()
    } else if let Some(rest) = name.strip_prefix('#') {
        char::from_u32(rest.parse::<u32>().ok()?)?.to_string()
    } else {
        named_entity(name)?.to_string()
    };
    Some((decoded, semi + 1))
}

fn named_entity(name: &str) -> Option<&'static str> {
    include!("entities_match.inc")
}

fn emphasis_open_allowed(s: &str, i: usize, marker_len: usize, marker: char) -> bool {
    let before = s[..i].chars().next_back();
    let after = s[i + marker_len..].chars().next();
    let after_ws = after.is_none_or(char::is_whitespace);
    let after_punct = after.is_some_and(|c| c.is_ascii_punctuation());
    let before_ws = before.is_none_or(char::is_whitespace);
    let before_punct = before.is_some_and(|c| c.is_ascii_punctuation());
    if after_ws {
        return false;
    }
    if marker == '_' && before.is_some_and(|c| c.is_alphanumeric()) && after.is_some_and(|c| c.is_alphanumeric()) {
        return false;
    }
    !after_punct || before_ws || before_punct
}

fn find_emphasis_close(s: &str, marker: char, marker_len: usize) -> Option<usize> {
    let needle: String = std::iter::repeat_n(marker, marker_len).collect();
    let mut from = 0usize;
    while let Some(rel) = s[from..].find(&needle) {
        let pos = from + rel;
        let before = s[..pos].chars().next_back();
        let after = s[pos + marker_len..].chars().next();
        let before_ws = before.is_none_or(char::is_whitespace);
        let before_punct = before.is_some_and(|c| c.is_ascii_punctuation());
        let after_ws = after.is_none_or(char::is_whitespace);
        let after_punct = after.is_some_and(|c| c.is_ascii_punctuation());
        let right_flanking = !before_ws && (!before_punct || after_ws || after_punct);
        let intraword_underscore =
            marker == '_' && before.is_some_and(|c| c.is_alphanumeric()) && after.is_some_and(|c| c.is_alphanumeric());
        if right_flanking && !intraword_underscore {
            return Some(pos);
        }
        from = pos + marker_len;
    }
    None
}

pub(super) fn strip_inline_markup_inner(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let tail = &s[i..];
        if tail.starts_with('\\') {
            if let Some(ch) = tail[1..].chars().next() {
                if ch.is_ascii_punctuation() {
                    match ch {
                        '&' => out.push_str("&amp;"),
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        _ => out.push(ch),
                    }
                    i += 1 + ch.len_utf8();
                    continue;
                }
            }
        }
        if tail.starts_with('$') && !tail.starts_with("$$") {
            if let Some(end) = tail[1..].find('$') {
                out.push_str(&tail[1..1 + end]);
                i += end + 2;
                continue;
            }
        }
        if tail.starts_with("~~") {
            i += 2;
            continue;
        }
        if tail.starts_with("**") || tail.starts_with("__") {
            i += 2;
            continue;
        }
        let Some(ch) = tail.chars().next() else { break };
        if matches!(ch, '*' | '_' | '`') {
            i += ch.len_utf8();
            continue;
        }
        if ch == '[' {
            if let Some(rb) = tail[1..].find(']') {
                let rb = rb + 1;
                if tail[rb + 1..].starts_with('(') {
                    if let Some(rp0) = tail[rb + 2..].find(')') {
                        out.push_str(&tail[1..rb]);
                        i += rb + 2 + rp0 + 1;
                        continue;
                    }
                }
            }
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

pub(super) fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

pub(super) fn escape_attr(s: &str) -> String {
    escape_markup(s).replace('"', "&quot;").replace('\'', "&apos;")
}
