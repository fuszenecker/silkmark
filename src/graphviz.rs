use crate::graph::{Direction, Edge, Graph, Node, Shape};
use std::collections::HashMap;

const MAX_SOURCE_LEN: usize = 256 * 1024;
const MAX_NODES: usize = 512;
const MAX_EDGES: usize = 2048;
const MAX_LABEL_CHARS: usize = 512;
const MAX_ID_CHARS: usize = 256;

fn clean_id(token: &str) -> String {
    token.trim().trim_matches('"').trim().to_string()
}

fn attr_value(attrs: &str, key: &str) -> Option<String> {
    let mut in_quote = false;
    let mut start = 0usize;
    let bytes = attrs.as_bytes();
    let mut parts = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => in_quote = !in_quote,
            b',' if !in_quote => {
                parts.push(attrs[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(attrs[start..].trim());
    for part in parts {
        let Some((k, v)) = part.split_once('=') else { continue };
        if k.trim().eq_ignore_ascii_case(key) {
            return Some(v.trim().trim_matches('"').replace("\\n", " ").chars().take(MAX_LABEL_CHARS).collect());
        }
    }
    None
}

fn parse_attrs(text: &str) -> (&str, Option<&str>) {
    let trimmed = text.trim();
    if let Some(open) = trimmed.find('[') {
        if let Some(close_rel) = trimmed[open + 1..].rfind(']') {
            let close = open + 1 + close_rel;
            return (trimmed[..open].trim(), Some(&trimmed[open + 1..close]));
        }
    }
    (trimmed, None)
}

fn shape_from_attr(value: Option<String>) -> Shape {
    match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("ellipse") | Some("circle") | Some("oval") => Shape::Circle,
        Some("diamond") => Shape::Diamond,
        Some("box") | Some("rect") | Some("rectangle") => Shape::Rectangle,
        Some("rounded") => Shape::Rounded,
        _ => Shape::Rectangle,
    }
}

fn split_statements(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut bracket_depth = 0u32;
    for ch in body.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                current.push(ch);
            }
            '[' if !in_quote => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' if !in_quote => {
                bracket_depth = bracket_depth.saturating_sub(1);
                current.push(ch);
            }
            ';' | '\n' if !in_quote && bracket_depth == 0 => {
                let s = current.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let s = current.trim();
    if !s.is_empty() {
        out.push(s.to_string());
    }
    out
}

fn strip_line_comment(line: &str) -> &str {
    let mut in_quote = false;
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'"' {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if !in_quote && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            return &line[..i];
        }
        i += 1;
    }
    line
}

pub fn parse(source: &str) -> Result<Graph, String> {
    if source.len() > MAX_SOURCE_LEN {
        return Err("Graphviz source is too large".into());
    }
    let src = source.trim();
    let lower = src.to_ascii_lowercase();
    let (directed_graph, header_len) = if lower.starts_with("strict digraph") {
        (true, "strict digraph".len())
    } else if lower.starts_with("digraph") {
        (true, "digraph".len())
    } else if lower.starts_with("strict graph") {
        (false, "strict graph".len())
    } else if lower.starts_with("graph") {
        (false, "graph".len())
    } else {
        return Err("only Graphviz graph/digraph documents are supported".into());
    };

    let open =
        src[header_len..].find('{').map(|p| p + header_len).ok_or_else(|| "Graphviz document is missing '{'".to_string())?;
    let close = src.rfind('}').ok_or_else(|| "Graphviz document is missing '}'".to_string())?;
    if close <= open {
        return Err("invalid Graphviz document body".into());
    }
    let body = &src[open + 1..close];

    let mut graph = Graph { direction: Direction::TopDown, nodes: Vec::new(), edges: Vec::new() };
    let mut indexes = HashMap::<String, usize>::new();

    fn ensure_node(
        graph: &mut Graph,
        indexes: &mut HashMap<String, usize>,
        token: &str,
        attrs: Option<&str>,
    ) -> Result<usize, String> {
        let id = clean_id(token);
        if id.is_empty() {
            return Err("empty Graphviz node id".into());
        }
        if id.chars().count() > MAX_ID_CHARS {
            return Err(format!("Graphviz node id exceeds {MAX_ID_CHARS} characters"));
        }
        let label = attrs.and_then(|a| attr_value(a, "label"));
        let shape = shape_from_attr(attrs.and_then(|a| attr_value(a, "shape")));
        if let Some(&idx) = indexes.get(&id) {
            if let Some(label) = label {
                graph.nodes[idx].label = label;
            }
            if attrs.and_then(|a| attr_value(a, "shape")).is_some() {
                graph.nodes[idx].shape = shape;
            }
            return Ok(idx);
        }
        if graph.nodes.len() >= MAX_NODES {
            return Err(format!("Graphviz diagram exceeds {MAX_NODES} nodes"));
        }
        let idx = graph.nodes.len();
        graph.nodes.push(Node { label: label.unwrap_or_else(|| id.clone()), shape });
        indexes.insert(id, idx);
        Ok(idx)
    }

    for statement in split_statements(body) {
        let statement = strip_line_comment(&statement).trim();
        if statement.is_empty() || statement.starts_with('#') {
            continue;
        }
        let lower_stmt = statement.to_ascii_lowercase();
        if lower_stmt.starts_with("rankdir") {
            if let Some((_, value)) = statement.split_once('=') {
                graph.direction = match value.trim().trim_matches('"').to_ascii_uppercase().as_str() {
                    "TB" | "TD" => Direction::TopDown,
                    "BT" => Direction::BottomUp,
                    "LR" => Direction::LeftRight,
                    "RL" => Direction::RightLeft,
                    _ => graph.direction,
                };
            }
            continue;
        }
        if lower_stmt.starts_with("node ")
            || lower_stmt.starts_with("edge ")
            || lower_stmt.starts_with("graph ")
            || lower_stmt.starts_with("subgraph ")
            || statement == "}"
        {
            continue;
        }

        let edge_op = if statement.contains("->") {
            Some(("->", true))
        } else if statement.contains("--") {
            Some(("--", false))
        } else {
            None
        };

        if let Some((op, directed_edge)) = edge_op {
            if directed_graph != directed_edge {
                return Err(format!("edge operator '{op}' does not match graph kind"));
            }
            let Some(pos) = statement.find(op) else { continue };
            let left = statement[..pos].trim();
            let (right_token, attrs) = parse_attrs(statement[pos + op.len()..].trim());
            let from = ensure_node(&mut graph, &mut indexes, left, None)?;
            let to = ensure_node(&mut graph, &mut indexes, right_token, None)?;
            let label = attrs.and_then(|a| attr_value(a, "label"));
            if graph.edges.len() >= MAX_EDGES {
                return Err(format!("Graphviz diagram exceeds {MAX_EDGES} edges"));
            }
            graph.edges.push(Edge { from, to, label, directed: directed_edge });
        } else if statement.contains('=') && !statement.contains('[') {
            continue;
        } else {
            let (token, attrs) = parse_attrs(statement);
            let _ = ensure_node(&mut graph, &mut indexes, token, attrs)?;
        }
    }

    if graph.nodes.is_empty() {
        return Err("Graphviz graph contains no nodes".into());
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_digraph_with_labels_and_shapes() {
        let g = parse(
            "digraph G { rankdir=LR; start [label=\"Start\", shape=box]; check [shape=diamond]; start -> check [label=\"next\"]; }",
        )
        .unwrap();
        assert_eq!(g.direction, Direction::LeftRight);
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].label.as_deref(), Some("next"));
        assert_eq!(g.nodes[1].shape, Shape::Diamond);
    }

    #[test]
    fn parses_undirected_graph() {
        let g = parse("graph G { a -- b; b -- c; }").unwrap();
        assert_eq!(g.nodes.len(), 3);
        assert!(g.edges.iter().all(|edge| !edge.directed));
    }

    #[test]
    fn rejects_wrong_edge_operator() {
        assert!(parse("digraph G { a -- b; }").is_err());
    }

    #[test]
    fn rejects_oversized_source() {
        let source = format!("digraph G {{ a [label=\"{}\"]; }}", "x".repeat(MAX_SOURCE_LEN));
        assert!(parse(&source).is_err());
    }
}
