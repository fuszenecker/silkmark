//! Tiny LaTeX-like math parser for SilkMark.
//! It intentionally supports a compact documentation-oriented subset.

pub fn render(expr: &str) -> String {
    Parser::new(expr).parse_until(None)
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn parse_until(&mut self, stop: Option<char>) -> String {
        let mut out = String::new();
        while self.i < self.s.len() {
            let Some(ch) = self.peek() else {
                break;
            };
            if Some(ch) == stop {
                self.bump();
                break;
            }
            match ch {
                '\\' => {
                    self.bump();
                    out.push_str(&self.command());
                }
                '^' => {
                    self.bump();
                    let x = self.argument();
                    out.push_str("<span rise=\"7000\" size=\"smaller\">");
                    out.push_str(&x);
                    out.push_str("</span>");
                }
                '_' => {
                    self.bump();
                    let x = self.argument();
                    out.push_str("<span rise=\"-3500\" size=\"smaller\">");
                    out.push_str(&x);
                    out.push_str("</span>");
                }
                '{' => {
                    self.bump();
                    out.push_str(&self.parse_until(Some('}')));
                }
                '}' if stop.is_none() => {
                    self.bump();
                    out.push('}');
                }
                _ => {
                    self.bump();
                    push_escaped(&mut out, ch);
                }
            }
        }
        out
    }

    fn command(&mut self) -> String {
        let start = self.i;
        while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.bump();
        }
        if self.i == start {
            if let Some(ch) = self.bump() {
                return escape_char(ch);
            }
            return String::new();
        }
        let name = &self.s[start..self.i];
        match name {
            "frac" => {
                let a = self.argument();
                let b = self.argument();
                // Compact stacked-looking fraction that works in a single Pango label.
                format!("<span rise=\"5200\" size=\"smaller\">{a}</span>⁄<span rise=\"-3200\" size=\"smaller\">{b}</span>")
            }
            "sqrt" => {
                let x = self.argument();
                format!("√<span underline=\"single\">{x}</span>")
            }
            "text" | "mathrm" | "operatorname" => self.argument(),
            "mathbf" => format!("<b>{}</b>", self.argument()),
            "mathit" => format!("<i>{}</i>", self.argument()),
            "vec" => format!("{}⃗", self.argument()),
            "hat" => format!("{}̂", self.argument()),
            "bar" | "overline" => format!("{}̅", self.argument()),
            "underline" => format!("<span underline=\"single\">{}</span>", self.argument()),
            "begin" => self.environment(),
            "left" | "right" => self.delimiter(),
            "lim" | "sin" | "cos" | "tan" | "cot" | "sec" | "csc" | "sinh" | "cosh" | "tanh" | "log" | "ln" | "exp" | "min"
            | "max" => {
                format!("<span font_family=\"serif\">{}</span>", name)
            }
            _ => symbol(name).map(str::to_string).unwrap_or_else(|| format!("\\{}", escape_text(name))),
        }
    }

    fn delimiter(&mut self) -> String {
        self.skip_spaces();
        if self.peek() == Some('.') {
            self.bump();
            return String::new();
        }
        if self.peek() == Some('\\') {
            self.bump();
            let start = self.i;
            while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                self.bump();
            }
            if self.i == start {
                return self.bump().map(escape_char).unwrap_or_default();
            }
            return match &self.s[start..self.i] {
                "langle" => "⟨".into(),
                "rangle" => "⟩".into(),
                "lvert" | "rvert" | "vert" => "|".into(),
                "lVert" | "rVert" | "Vert" => "‖".into(),
                "lbrace" => "{".into(),
                "rbrace" => "}".into(),
                other => symbol(other).map(str::to_string).unwrap_or_else(|| format!("\\{}", escape_text(other))),
            };
        }
        self.bump().map(escape_char).unwrap_or_default()
    }

    fn argument(&mut self) -> String {
        self.skip_spaces();
        match self.peek() {
            Some('{') => {
                self.bump();
                self.parse_until(Some('}'))
            }
            Some('\\') => {
                self.bump();
                self.command()
            }
            Some(_) => self.bump().map(escape_char).unwrap_or_default(),
            None => String::new(),
        }
    }

    fn raw_braced(&mut self) -> String {
        self.skip_spaces();
        if self.peek() != Some('{') {
            return String::new();
        }
        self.bump();
        let start = self.i;
        let mut depth = 1usize;
        while self.i < self.s.len() {
            match self.bump() {
                Some('{') => depth += 1,
                Some('}') => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.i - 1;
                        return self.s[start..end].to_string();
                    }
                }
                Some(_) => {}
                None => break,
            }
        }
        self.s[start..].to_string()
    }

    fn environment(&mut self) -> String {
        let env = self.raw_braced();
        if !matches!(env.as_str(), "matrix" | "pmatrix" | "bmatrix" | "cases") {
            return format!("\\begin{{{}}}", escape_text(&env));
        }
        let end_marker = format!("\\end{{{}}}", env);
        let rest = &self.s[self.i..];
        let Some(pos) = rest.find(&end_marker) else {
            return format!("\\begin{{{}}}", escape_text(&env));
        };
        let body = rest[..pos].trim();
        self.i += pos + end_marker.len();

        let rows: Vec<Vec<String>> = body.split("\\\\").map(|row| row.split('&').map(|c| render(c.trim())).collect()).collect();

        if env == "cases" {
            let mut lines = Vec::new();
            for cells in rows {
                let lhs = cells.first().cloned().unwrap_or_default();
                let rhs = cells.get(1).cloned().unwrap_or_default();
                if rhs.is_empty() {
                    lines.push(lhs);
                } else {
                    lines.push(format!("{}    {}", lhs, rhs));
                }
            }
            return format!("{{ {}", lines.join("\n  "));
        }

        let inner = rows.into_iter().map(|cells| cells.join("   ")).collect::<Vec<_>>().join("\n");
        match env.as_str() {
            "pmatrix" => format!("( {} )", inner),
            "bmatrix" => format!("[ {} ]", inner),
            _ => inner,
        }
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(' ') {
            self.bump();
        }
    }
    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += ch.len_utf8();
        Some(ch)
    }
}

fn symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "varepsilon" => "ϵ",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "ϑ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "rho" => "ρ",
        "sigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "varphi" => "ϕ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "oint" => "∮",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "pm" => "±",
        "mp" => "∓",
        "times" => "×",
        "cdot" => "·",
        "div" => "÷",
        "le" | "leq" => "≤",
        "ge" | "geq" => "≥",
        "ne" | "neq" => "≠",
        "approx" => "≈",
        "equiv" => "≡",
        "to" | "rightarrow" => "→",
        "leftarrow" => "←",
        "leftrightarrow" => "↔",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "in" => "∈",
        "notin" => "∉",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "supseteq" => "⊇",
        "cup" => "∪",
        "cap" => "∩",
        "forall" => "∀",
        "exists" => "∃",
        "neg" => "¬",
        "land" => "∧",
        "lor" => "∨",
        "ldots" => "…",
        "cdots" => "⋯",
        "degree" => "°",
        "langle" => "⟨",
        "rangle" => "⟩",
        _ => return None,
    })
}

fn push_escaped(out: &mut String, ch: char) {
    out.push_str(&escape_char(ch));
}
fn escape_char(ch: char) -> String {
    match ch {
        '&' => "&amp;".into(),
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        _ => ch.to_string(),
    }
}
fn escape_text(s: &str) -> String {
    s.chars().map(escape_char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn greek_and_indices() {
        let x = render(r"\alpha_i^2");
        assert!(x.contains("α"));
        assert!(x.contains("rise=\"-3500\""));
        assert!(x.contains("rise=\"7000\""));
    }
    #[test]
    fn fraction_and_root() {
        let x = render(r"\frac{a+b}{\sqrt{x}}");
        assert!(x.contains('⁄'));
        assert!(x.contains('√'));
    }
    #[test]
    fn comparisons() {
        assert!(render(r"x \le y \rightarrow \infty").contains("≤"));
    }
    #[test]
    fn accents() {
        let x = render(r"\vec{x} \hat{y} \bar{z}");
        assert!(x.contains('⃗'));
        assert!(x.contains('̂'));
        assert!(x.contains('̅'));
    }
    #[test]
    fn matrix() {
        let x = render(r"\begin{matrix}a & b \\ c & d\end{matrix}");
        assert!(x.contains("a   b"));
        assert!(x.contains("c   d"));
    }
    #[test]
    fn cases_environment() {
        let x = render(r"\begin{cases}x^2 & x \ge 0 \\ -x & x < 0\end{cases}");
        assert!(x.starts_with('{'));
        assert!(x.contains('≤') == false);
        assert!(x.contains('≥'));
        assert!(x.contains('\n'));
    }
    #[test]
    fn named_operators() {
        let x = render(r"\lim_{x\to0} \frac{\sin x}{x}");
        assert!(x.contains("lim"));
        assert!(x.contains("sin"));
        assert!(x.contains('→'));
    }
    #[test]
    fn left_right_delimiters() {
        let x = render(r"\left( x+1 \right) \left\langle y \right\rangle");
        assert!(x.contains('('));
        assert!(x.contains(')'));
        assert!(x.contains('⟨'));
        assert!(x.contains('⟩'));
    }
    #[test]
    fn invisible_delimiter() {
        let x = render(r"\left. x \right|");
        assert!(!x.contains('.'));
        assert!(x.contains('|'));
    }
}
