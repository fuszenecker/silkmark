use crate::ffi::*;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::ffi::CString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    TopDown,
    BottomUp,
    LeftRight,
    RightLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Rectangle,
    Rounded,
    Diamond,
    Circle,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub label: String,
    pub shape: Shape,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
    pub directed: bool,
}

#[derive(Clone, Debug)]
pub struct Graph {
    pub direction: Direction,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Copy)]
struct BoxRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn node_size(node: &Node) -> (f64, f64) {
    let chars = node.label.chars().count().min(48) as f64;
    let w = (chars * 7.4 + 36.0).clamp(92.0, 360.0);
    let h = if node.shape == Shape::Diamond { 68.0 } else { 48.0 };
    if node.shape == Shape::Circle {
        let d = w.max(h).min(150.0);
        (d, d)
    } else {
        (w, h)
    }
}

fn layers(graph: &Graph) -> Vec<Vec<usize>> {
    let n = graph.nodes.len();
    let mut indegree = vec![0usize; n];
    let mut outgoing = vec![Vec::<usize>::new(); n];
    for edge in &graph.edges {
        outgoing[edge.from].push(edge.to);
        indegree[edge.to] = indegree[edge.to].saturating_add(1);
    }
    let mut q = VecDeque::new();
    for (i, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            q.push_back(i);
        }
    }
    if q.is_empty() {
        q.push_back(0);
    }
    let mut depth = vec![0usize; n];
    let mut seen = vec![false; n];
    while let Some(u) = q.pop_front() {
        seen[u] = true;
        for &v in &outgoing[u] {
            depth[v] = depth[v].max(depth[u] + 1);
            indegree[v] = indegree[v].saturating_sub(1);
            if indegree[v] == 0 {
                q.push_back(v);
            }
        }
    }
    let max_seen = depth.iter().copied().max().unwrap_or(0);
    for i in 0..n {
        if !seen[i] {
            depth[i] = max_seen + 1;
        }
    }
    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let mut result = vec![Vec::new(); max_depth + 1];
    for (i, d) in depth.into_iter().enumerate() {
        result[d].push(i);
    }
    result.into_iter().filter(|l| !l.is_empty()).collect()
}

fn layout(graph: &Graph) -> (Vec<BoxRect>, i32, i32) {
    const MARGIN: f64 = 28.0;
    const GAP_X: f64 = 44.0;
    const GAP_Y: f64 = 64.0;
    let layer_list = layers(graph);
    let vertical = matches!(graph.direction, Direction::TopDown | Direction::BottomUp);
    let mut rects = vec![BoxRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }; graph.nodes.len()];
    let mut layer_major = 0.0f64;
    let mut canvas_cross = 0.0f64;

    let layer_cross_sizes: Vec<f64> = layer_list
        .iter()
        .map(|layer| {
            layer
                .iter()
                .enumerate()
                .map(|(j, &idx)| {
                    let (w, h) = node_size(&graph.nodes[idx]);
                    (if vertical { w } else { h })
                        + if j == 0 {
                            0.0
                        } else if vertical {
                            GAP_X
                        } else {
                            GAP_Y
                        }
                })
                .sum::<f64>()
        })
        .collect();
    for size in &layer_cross_sizes {
        canvas_cross = canvas_cross.max(*size);
    }

    for (li, layer) in layer_list.iter().enumerate() {
        let max_major = layer
            .iter()
            .map(|&idx| {
                let (w, h) = node_size(&graph.nodes[idx]);
                if vertical { h } else { w }
            })
            .fold(0.0f64, f64::max);
        let mut cross = MARGIN + (canvas_cross - layer_cross_sizes[li]) / 2.0;
        for &idx in layer {
            let (w, h) = node_size(&graph.nodes[idx]);
            if vertical {
                rects[idx] = BoxRect { x: cross, y: MARGIN + layer_major, w, h };
                cross += w + GAP_X;
            } else {
                rects[idx] = BoxRect { x: MARGIN + layer_major, y: cross, w, h };
                cross += h + GAP_Y;
            }
        }
        layer_major += max_major + if vertical { GAP_Y } else { GAP_X };
    }

    let (mut width, mut height) = if vertical {
        (canvas_cross + MARGIN * 2.0, layer_major - GAP_Y + MARGIN * 2.0)
    } else {
        (layer_major - GAP_X + MARGIN * 2.0, canvas_cross + MARGIN * 2.0)
    };
    width = width.max(260.0);
    height = height.max(150.0);

    if matches!(graph.direction, Direction::BottomUp) {
        for r in &mut rects {
            r.y = height - MARGIN - r.h - (r.y - MARGIN);
        }
    } else if matches!(graph.direction, Direction::RightLeft) {
        for r in &mut rects {
            r.x = width - MARGIN - r.w - (r.x - MARGIN);
        }
    }

    (rects, width.ceil() as i32, height.ceil() as i32)
}

pub fn preferred_size(graph: &Graph) -> (i32, i32) {
    let (_, w, h) = layout(graph);
    (w, h)
}

unsafe fn draw_text(cr: *mut cairo_t, text: &str, x: f64, y: f64, size: f64) {
    let Ok(font) = CString::new("Sans") else {
        return;
    };
    let Ok(text) = CString::new(text.replace('\0', "")) else {
        return;
    };
    cairo_select_font_face(cr, font.as_ptr(), 0, 0);
    cairo_set_font_size(cr, size);
    let mut ext = CairoTextExtents { x_bearing: 0.0, y_bearing: 0.0, width: 0.0, height: 0.0, x_advance: 0.0, y_advance: 0.0 };
    cairo_text_extents(cr, text.as_ptr(), &mut ext as *mut CairoTextExtents);
    cairo_move_to(cr, x - ext.width / 2.0 - ext.x_bearing, y - ext.height / 2.0 - ext.y_bearing);
    cairo_show_text(cr, text.as_ptr());
}

unsafe fn draw_node(cr: *mut cairo_t, node: &Node, r: BoxRect) {
    cairo_set_line_width(cr, 1.5);
    cairo_set_source_rgba(cr, 0.18, 0.20, 0.23, 1.0);
    match node.shape {
        Shape::Rectangle | Shape::Rounded => {
            cairo_rectangle(cr, r.x, r.y, r.w, r.h);
        }
        Shape::Diamond => {
            cairo_move_to(cr, r.x + r.w / 2.0, r.y);
            cairo_line_to(cr, r.x + r.w, r.y + r.h / 2.0);
            cairo_line_to(cr, r.x + r.w / 2.0, r.y + r.h);
            cairo_line_to(cr, r.x, r.y + r.h / 2.0);
            cairo_close_path(cr);
        }
        Shape::Circle => {
            cairo_arc(cr, r.x + r.w / 2.0, r.y + r.h / 2.0, r.w.min(r.h) / 2.0, 0.0, PI * 2.0);
        }
    }
    cairo_set_source_rgba(cr, 0.96, 0.96, 0.97, 1.0);
    cairo_fill_preserve(cr);
    cairo_set_source_rgba(cr, 0.22, 0.24, 0.27, 1.0);
    cairo_stroke(cr);
    draw_text(cr, &node.label, r.x + r.w / 2.0, r.y + r.h / 2.0, 13.0);
}

fn center(r: BoxRect) -> (f64, f64) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

unsafe fn draw_edge(cr: *mut cairo_t, edge: &Edge, a: BoxRect, b: BoxRect) {
    let (ax, ay) = center(a);
    let (bx, by) = center(b);
    let dx = bx - ax;
    let dy = by - ay;
    let horizontal = dx.abs() > dy.abs();
    let (sx, sy, ex, ey) = if horizontal {
        let sign = if dx >= 0.0 { 1.0 } else { -1.0 };
        (ax + sign * a.w / 2.0, ay, bx - sign * b.w / 2.0, by)
    } else {
        let sign = if dy >= 0.0 { 1.0 } else { -1.0 };
        (ax, ay + sign * a.h / 2.0, bx, by - sign * b.h / 2.0)
    };
    cairo_set_line_width(cr, 1.4);
    cairo_set_source_rgba(cr, 0.32, 0.34, 0.38, 1.0);
    cairo_move_to(cr, sx, sy);
    if horizontal {
        let mx = (sx + ex) / 2.0;
        cairo_line_to(cr, mx, sy);
        cairo_line_to(cr, mx, ey);
    } else {
        let my = (sy + ey) / 2.0;
        cairo_line_to(cr, sx, my);
        cairo_line_to(cr, ex, my);
    }
    cairo_line_to(cr, ex, ey);
    cairo_stroke(cr);

    if edge.directed {
        let angle = (ey - sy).atan2(ex - sx);
        let len = 9.0;
        let spread = 0.55;
        cairo_move_to(cr, ex, ey);
        cairo_line_to(cr, ex - len * (angle - spread).cos(), ey - len * (angle - spread).sin());
        cairo_move_to(cr, ex, ey);
        cairo_line_to(cr, ex - len * (angle + spread).cos(), ey - len * (angle + spread).sin());
        cairo_stroke(cr);
    }
    if let Some(label) = &edge.label {
        draw_text(cr, label, (sx + ex) / 2.0, (sy + ey) / 2.0 - 8.0, 11.0);
    }
}

pub unsafe fn draw(graph: &Graph, cr: *mut cairo_t, width: i32, height: i32) {
    let (rects, natural_w, natural_h) = layout(graph);
    let sx = width as f64 / natural_w.max(1) as f64;
    let sy = height as f64 / natural_h.max(1) as f64;
    let scale = sx.min(sy);
    cairo_save(cr);
    cairo_translate(cr, (width as f64 - natural_w as f64 * scale) / 2.0, 0.0);
    cairo_scale(cr, scale, scale);
    for edge in &graph.edges {
        draw_edge(cr, edge, rects[edge.from], rects[edge.to]);
    }
    for (idx, node) in graph.nodes.iter().enumerate() {
        draw_node(cr, node, rects[idx]);
    }
    cairo_restore(cr);
}
