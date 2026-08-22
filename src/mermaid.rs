use crate::graph::{Direction, Edge, Graph, Node, Shape};
use std::collections::HashMap;

const MAX_SOURCE_LEN: usize = 256 * 1024;
const MAX_NODES: usize = 512;
const MAX_EDGES: usize = 2048;
const MAX_LABEL_CHARS: usize = 512;
const MAX_ID_CHARS: usize = 256;

fn clean_label(s: &str) -> String {
    s.trim().trim_matches('"').replace("<br/>", " ").replace("<br>", " ").chars().take(MAX_LABEL_CHARS).collect()
}

fn parse_node_token(token: &str) -> Option<(String, Option<String>, Option<Shape>)> {
    let t = token.trim().trim_end_matches(';').trim();
    if t.is_empty() {
        return None;
    }
    let id_end =
        t.char_indices().find_map(|(i, c)| (!c.is_ascii_alphanumeric() && c != '_' && c != '-').then_some(i)).unwrap_or(t.len());
    if id_end == 0 {
        return None;
    }
    let id = t[..id_end].to_string();
    let rest = t[id_end..].trim();
    if rest.is_empty() {
        return Some((id, None, None));
    }
    if rest.starts_with("((") && rest.ends_with("))") && rest.len() >= 4 {
        return Some((id, Some(clean_label(&rest[2..rest.len() - 2])), Some(Shape::Circle)));
    }
    if rest.starts_with('[') && rest.ends_with(']') && rest.len() >= 2 {
        return Some((id, Some(clean_label(&rest[1..rest.len() - 1])), Some(Shape::Rectangle)));
    }
    if rest.starts_with('{') && rest.ends_with('}') && rest.len() >= 2 {
        return Some((id, Some(clean_label(&rest[1..rest.len() - 1])), Some(Shape::Diamond)));
    }
    if rest.starts_with('(') && rest.ends_with(')') && rest.len() >= 2 {
        return Some((id, Some(clean_label(&rest[1..rest.len() - 1])), Some(Shape::Rounded)));
    }
    Some((id, None, None))
}

fn edge_parts(line: &str) -> Option<(&str, &str, Option<String>, bool)> {
    if let Some(pos) = line.find("-->") {
        let left = &line[..pos];
        let mut right = line[pos + 3..].trim_start();
        let mut label = None;
        if let Some(rest) = right.strip_prefix('|') {
            if let Some(end) = rest.find('|') {
                label = Some(clean_label(&rest[..end]));
                right = rest[end + 1..].trim_start();
            }
        }
        return Some((left, right, label, true));
    }
    if let Some(pos) = line.find("---") {
        return Some((&line[..pos], line[pos + 3..].trim_start(), None, false));
    }
    None
}

pub fn parse(source: &str) -> Result<Graph, String> {
    if source.len() > MAX_SOURCE_LEN {
        return Err("Mermaid diagram source is too large".into());
    }
    let mut lines = source.lines().map(str::trim).filter(|l| !l.is_empty());
    let header = lines.next().ok_or_else(|| "empty Mermaid diagram".to_string())?;
    let mut header_parts = header.split_whitespace();
    let kind = header_parts.next().unwrap_or_default();
    if !kind.eq_ignore_ascii_case("flowchart") && !kind.eq_ignore_ascii_case("graph") {
        return Err("only Mermaid flowchart/graph diagrams are supported".into());
    }
    let direction = match header_parts.next().unwrap_or("TD").to_ascii_uppercase().as_str() {
        "TD" | "TB" => Direction::TopDown,
        "BT" => Direction::BottomUp,
        "LR" => Direction::LeftRight,
        "RL" => Direction::RightLeft,
        other => return Err(format!("unsupported Mermaid direction: {other}")),
    };

    let mut nodes = Vec::<Node>::new();
    let mut indexes = HashMap::<String, usize>::new();
    let mut edges = Vec::<Edge>::new();

    fn ensure_node(nodes: &mut Vec<Node>, indexes: &mut HashMap<String, usize>, token: &str) -> Result<usize, String> {
        let (id, label, shape) = parse_node_token(token).ok_or_else(|| format!("invalid Mermaid node: {token}"))?;
        if id.chars().count() > MAX_ID_CHARS {
            return Err(format!("Mermaid node id exceeds {MAX_ID_CHARS} characters"));
        }
        if let Some(&idx) = indexes.get(&id) {
            if let Some(label) = label {
                nodes[idx].label = label;
            }
            if let Some(shape) = shape {
                nodes[idx].shape = shape;
            }
            return Ok(idx);
        }
        if nodes.len() >= MAX_NODES {
            return Err(format!("Mermaid diagram exceeds {MAX_NODES} nodes"));
        }
        let idx = nodes.len();
        nodes.push(Node { label: label.unwrap_or_else(|| id.clone()), shape: shape.unwrap_or(Shape::Rectangle) });
        indexes.insert(id, idx);
        Ok(idx)
    }

    for raw in lines {
        let line = raw.trim_end_matches(';').trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        if line.starts_with("subgraph ") || line.eq_ignore_ascii_case("end") {
            continue;
        }
        if let Some((left, right, label, directed)) = edge_parts(line) {
            let from = ensure_node(&mut nodes, &mut indexes, left)?;
            let to = ensure_node(&mut nodes, &mut indexes, right)?;
            if edges.len() >= MAX_EDGES {
                return Err(format!("Mermaid diagram exceeds {MAX_EDGES} edges"));
            }
            edges.push(Edge { from, to, label, directed });
        } else {
            let _ = ensure_node(&mut nodes, &mut indexes, line)?;
        }
    }

    if nodes.is_empty() {
        return Err("Mermaid flowchart contains no nodes".into());
    }
    Ok(Graph { direction, nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_flowchart() {
        let g = parse("flowchart TD\nA[Start] --> B{Valid?}\nB -->|Yes| C[Done]").unwrap();
        assert_eq!(g.nodes.len(), 3);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.edges[1].label.as_deref(), Some("Yes"));
        assert_eq!(g.nodes[1].shape, Shape::Diamond);
    }

    #[test]
    fn parses_left_right_and_circle() {
        let g = parse("graph LR\nA((One)) --> B(Two)").unwrap();
        assert_eq!(g.direction, Direction::LeftRight);
        assert_eq!(g.nodes[0].shape, Shape::Circle);
        assert_eq!(g.nodes[1].shape, Shape::Rounded);
    }

    #[test]
    fn rejects_non_flowchart_diagram() {
        assert!(parse("sequenceDiagram\nA->>B: hello").is_err());
    }

    #[test]
    fn rejects_oversized_source() {
        let source = format!("flowchart TD\nA[{}]", "x".repeat(MAX_SOURCE_LEN));
        assert!(parse(&source).is_err());
    }
}
