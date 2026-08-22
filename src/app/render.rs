use super::*;

struct GraphViewCtx {
    graph: crate::graph::Graph,
    area: *mut GtkWidget,
    scrolled: *mut GtkWidget,
    natural_w: i32,
    natural_h: i32,
    zoom: f64,
}

unsafe fn apply_graph_zoom(ctx: &mut GraphViewCtx, zoom: f64) {
    ctx.zoom = zoom.clamp(0.25, 2.0);
    let width = ((ctx.natural_w as f64) * ctx.zoom).round() as i32;
    let height = ((ctx.natural_h as f64) * ctx.zoom).round() as i32;
    let drawing_width = width.max(180);
    let drawing_height = height.max(100);
    gtk_widget_set_size_request(ctx.area, drawing_width, drawing_height);

    // GtkScrolledWindow tends to collapse to a very small natural height when its
    // child is a GtkDrawingArea. Give diagram viewports an explicit content-sized
    // height, capped so genuinely large diagrams still scroll inside the document.
    if !ctx.scrolled.is_null() {
        const MAX_DIAGRAM_VIEWPORT_HEIGHT: i32 = 720;
        const VIEWPORT_PADDING: i32 = 16;
        let viewport_height = drawing_height.saturating_add(VIEWPORT_PADDING).min(MAX_DIAGRAM_VIEWPORT_HEIGHT);
        gtk_widget_set_size_request(ctx.scrolled, -1, viewport_height);
    }

    gtk_widget_queue_draw(ctx.area);
}

unsafe extern "C" fn draw_graph(_area: *mut GtkDrawingArea, cr: *mut cairo_t, width: i32, height: i32, data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let ctx = &*(data as *const GraphViewCtx);
    crate::graph::draw(&ctx.graph, cr, width, height);
}

unsafe extern "C" fn graph_zoom_out(_button: *mut GtkButton, data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let ctx = &mut *(data as *mut GraphViewCtx);
    apply_graph_zoom(ctx, ctx.zoom / 1.25);
}

unsafe extern "C" fn graph_zoom_reset(_button: *mut GtkButton, data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let ctx = &mut *(data as *mut GraphViewCtx);
    apply_graph_zoom(ctx, 1.0);
}

unsafe extern "C" fn graph_zoom_in(_button: *mut GtkButton, data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let ctx = &mut *(data as *mut GraphViewCtx);
    apply_graph_zoom(ctx, ctx.zoom * 1.25);
}

unsafe extern "C" fn graph_zoom_fit(_button: *mut GtkButton, data: *mut c_void) {
    if data.is_null() {
        return;
    }
    let ctx = &mut *(data as *mut GraphViewCtx);
    let fit = (1000.0 / ctx.natural_w.max(1) as f64).min(700.0 / ctx.natural_h.max(1) as f64).min(1.0);
    apply_graph_zoom(ctx, fit);
}

unsafe extern "C" fn drop_graph_view(data: *mut c_void) {
    if !data.is_null() {
        drop(Box::from_raw(data as *mut GraphViewCtx));
    }
}

impl App {
    pub(crate) unsafe fn render_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let render_started = Instant::now();
        let page = self.tabs[idx].page;
        self.clear_animations_for_page(page);
        set_label(self.tabs[idx].tab_label, &short(&self.tabs[idx].title, 24));

        // A link/button inside the current document can still own keyboard focus
        // when navigation completes. Removing that focused subtree immediately can
        // leave GTK with a stale focus-chain pointer until its next main-loop
        // update. Clear window focus first, but only when the focused widget is
        // actually inside this document subtree.
        let focused = gtk_window_get_focus(self.window as *mut GtkWindow);
        if !focused.is_null() {
            let mut node = focused;
            let mut inside_document = false;
            while !node.is_null() {
                if node == self.tabs[idx].doc_box {
                    inside_document = true;
                    break;
                }
                node = gtk_widget_get_parent(node);
            }
            if inside_document {
                gtk_window_set_focus(self.window as *mut GtkWindow, ptr::null_mut());
            }
        }

        loop {
            let child = gtk_widget_get_first_child(self.tabs[idx].doc_box);
            if child.is_null() {
                break;
            }
            gtk_box_remove(self.tabs[idx].doc_box as *mut GtkBox, child);
        }
        let target_anchor = md::fragment_id(&self.tabs[idx].url);
        let lines = self.tabs[idx].doc.lines.clone();
        let search = self.tabs[idx].search.to_lowercase();
        let wanted_hit = self.tabs[idx].search_index;
        let mut found_hits = 0usize;
        let mut search_widget: *mut GtkWidget = ptr::null_mut();
        let mut anchor_widget: *mut GtkWidget = ptr::null_mut();
        let mut line_i = 0usize;
        while line_i < lines.len() {
            let line = lines[line_i].clone();
            if matches!(line.style, Style::TableRow)
                && line_i + 1 < lines.len()
                && matches!(lines[line_i + 1].style, Style::TableSep)
            {
                let header = line.clone();
                let align = lines[line_i + 1].table.as_ref().map(|t| t.align.clone()).unwrap_or_default();
                let mut rows = vec![header];
                line_i += 2;
                while line_i < lines.len() && matches!(lines[line_i].style, Style::TableRow) {
                    rows.push(lines[line_i].clone());
                    line_i += 1;
                }

                let sc = gtk_scrolled_window_new();
                gtk_scrolled_window_set_policy(sc as *mut GtkScrolledWindow, GTK_POLICY_AUTOMATIC, GTK_POLICY_NEVER);
                gtk_widget_add_css_class(sc, c("md-table-scroll").as_ptr());
                let grid = gtk_grid_new();
                gtk_widget_add_css_class(grid, c("md-table").as_ptr());
                gtk_grid_set_row_spacing(grid as *mut GtkGrid, 1);
                gtk_grid_set_column_spacing(grid as *mut GtkGrid, 1);

                for (r, row) in rows.iter().enumerate() {
                    let cells = row.table.as_ref().map(|t| t.cells.clone()).unwrap_or_default();
                    for (col, cell) in cells.iter().enumerate() {
                        let label = gtk_label_new(ptr::null());
                        let (markup, _) = md::parse_inline_public(cell);
                        gtk_label_set_markup(label as *mut GtkLabel, c(&markup).as_ptr());
                        gtk_label_set_wrap(label as *mut GtkLabel, 1);
                        gtk_label_set_selectable(label as *mut GtkLabel, 1);
                        gtk_label_set_width_chars(label as *mut GtkLabel, 8);
                        gtk_label_set_max_width_chars(label as *mut GtkLabel, 48);
                        let xa = match align.get(col).copied().unwrap_or(TableAlign::Left) {
                            TableAlign::Left => 0.0,
                            TableAlign::Center => 0.5,
                            TableAlign::Right => 1.0,
                        };
                        gtk_label_set_xalign(label as *mut GtkLabel, xa);
                        gtk_widget_set_hexpand(label, 1);
                        gtk_widget_add_css_class(label, c(if r == 0 { "md-table-header" } else { "md-table-cell" }).as_ptr());
                        connect(label, "activate-link", on_inline_link as *const c_void, self as *mut App as *mut c_void, None);
                        if !search.is_empty() && md::strip_inline_markup(cell).to_lowercase().contains(&search) {
                            gtk_widget_add_css_class(label, c("search-hit").as_ptr());
                            if found_hits == wanted_hit {
                                gtk_widget_add_css_class(label, c("search-current").as_ptr());
                                search_widget = label;
                            }
                            found_hits += 1;
                        }
                        gtk_grid_attach(grid as *mut GtkGrid, label, col as i32, r as i32, 1, 1);
                    }
                }
                gtk_scrolled_window_set_child(sc as *mut GtkScrolledWindow, grid);
                let table_anchor_match = target_anchor
                    .as_ref()
                    .map(|a| rows.iter().any(|r| r.anchor.as_ref() == Some(a) || r.extra_anchors.iter().any(|x| x == a)))
                    .unwrap_or(false);
                if table_anchor_match {
                    gtk_widget_set_focusable(sc, 1);
                    anchor_widget = sc;
                }
                gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, sc);
                continue;
            }
            line_i += 1;
            if matches!(line.style, Style::Image) {
                if let Some(image) = line.image {
                    let wrap = gtk_box_new(GTK_ORIENTATION_VERTICAL, 3);
                    gtk_widget_add_css_class(wrap, c("md-image-wrap").as_ptr());
                    let picture = gtk_picture_new();
                    gtk_picture_set_can_shrink(picture as *mut GtkPicture, 1);
                    gtk_picture_set_keep_aspect_ratio(picture as *mut GtkPicture, 1);
                    gtk_widget_set_hexpand(picture, 1);
                    gtk_widget_set_size_request(picture, 1, 1);
                    gtk_picture_set_alternative_text(picture as *mut GtkPicture, c(&image.label).as_ptr());
                    if let Some(tip) = &image.title {
                        gtk_widget_set_tooltip_text(picture, c(tip).as_ptr());
                    }
                    let caption_text = if image.label.is_empty() {
                        "Loading image…".to_string()
                    } else {
                        format!("Loading image: {}", image.label)
                    };
                    let caption = gtk_label_new(c(&caption_text).as_ptr());
                    gtk_label_set_xalign(caption as *mut GtkLabel, 0.0);
                    gtk_widget_add_css_class(caption, c("md-image-caption").as_ptr());
                    gtk_box_append(wrap as *mut GtkBox, picture);
                    gtk_box_append(wrap as *mut GtkBox, caption);
                    gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, wrap);
                    if let Some(url) = net::resolve_resource(&self.tabs[idx].url, &image.target, false) {
                        if let Some(bytes) = self.image_cache_get(&url) {
                            if self.verbose {
                                eprintln!("[cache:image] {url}");
                            }
                            match set_picture_bytes(picture, &bytes) {
                                Ok(Some((loader, iter))) => {
                                    g_object_ref(picture as *mut c_void);
                                    self.animations.push(AnimatedImage { page: self.tabs[idx].page, picture, loader, iter });
                                    attach_zoom(picture, bytes, image.label.clone());
                                    gtk_widget_set_visible(caption, 0);
                                }
                                Ok(None) => {
                                    attach_zoom(picture, bytes, image.label.clone());
                                    gtk_widget_set_visible(caption, 0);
                                }
                                Err(e) => set_label(caption, &format!("[image error: {e}]")),
                            }
                        } else {
                            g_object_ref(picture as *mut c_void);
                            g_object_ref(caption as *mut c_void);
                            self.queued_images.push_back(QueuedImage {
                                page: self.tabs[idx].page,
                                picture,
                                caption,
                                url,
                                label: image.label,
                            });
                            if self.verbose {
                                eprintln!(
                                    "[queue:image] queued={} active={}",
                                    self.queued_images.len(),
                                    self.pending_images.len()
                                );
                            }
                            self.pump_image_queue();
                        }
                    } else {
                        set_label(caption, "[unsupported image location]");
                    }
                }
                continue;
            }
            if matches!(line.style, Style::Rule) {
                let rule = gtk_label_new(c("────────────────────────────────────────────────────────").as_ptr());
                gtk_widget_add_css_class(rule, c("md-rule").as_ptr());
                gtk_label_set_xalign(rule as *mut GtkLabel, 0.0);
                gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, rule);
                continue;
            }
            if matches!(line.style, Style::Diagram) {
                let Some(graph) = line.diagram.clone() else {
                    continue;
                };
                let source = line.code.as_ref().map(|b| b.text.clone()).unwrap_or_default();
                let frame = gtk_box_new(GTK_ORIENTATION_VERTICAL, 3);
                gtk_widget_add_css_class(frame, c("md-code-block").as_ptr());

                let head = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
                gtk_widget_add_css_class(head, c("md-code-head").as_ptr());
                let diagram_title = match line.code.as_ref().map(|b| b.language.to_ascii_lowercase()).as_deref() {
                    Some("dot") | Some("graphviz") | Some("gv") => "Graphviz/DOT graph",
                    _ => "Mermaid flowchart",
                };
                let title = gtk_label_new(c(diagram_title).as_ptr());
                gtk_label_set_xalign(title as *mut GtkLabel, 0.0);
                gtk_widget_set_hexpand(title, 1);
                let copy = gtk_button_new_with_label(c("Copy source").as_ptr());
                let ctx = Box::new(CopyCodeCtx { app: self as *mut App, text: source.clone() });
                connect(
                    copy,
                    "clicked",
                    on_copy_code as *const c_void,
                    Box::into_raw(ctx) as *mut c_void,
                    Some(drop_copy_code_ctx),
                );
                gtk_box_append(head as *mut GtkBox, title);
                gtk_box_append(head as *mut GtkBox, copy);
                gtk_box_append(frame as *mut GtkBox, head);

                let zoom_out = gtk_button_new_with_label(c("−").as_ptr());
                let zoom_reset = gtk_button_new_with_label(c("100%").as_ptr());
                let zoom_in = gtk_button_new_with_label(c("+").as_ptr());
                let zoom_fit = gtk_button_new_with_label(c("Fit").as_ptr());
                gtk_box_append(head as *mut GtkBox, zoom_out);
                gtk_box_append(head as *mut GtkBox, zoom_reset);
                gtk_box_append(head as *mut GtkBox, zoom_in);
                gtk_box_append(head as *mut GtkBox, zoom_fit);

                let (natural_w, natural_h) = crate::graph::preferred_size(&graph);
                let area = gtk_drawing_area_new();
                let sc = gtk_scrolled_window_new();
                let initial_zoom = (1000.0 / natural_w.max(1) as f64).min(700.0 / natural_h.max(1) as f64).min(1.0);
                let graph_ctx = Box::new(GraphViewCtx { graph, area, scrolled: sc, natural_w, natural_h, zoom: initial_zoom });
                let graph_data = Box::into_raw(graph_ctx);
                apply_graph_zoom(&mut *graph_data, initial_zoom);
                connect(zoom_out, "clicked", graph_zoom_out as *const c_void, graph_data as *mut c_void, None);
                connect(zoom_reset, "clicked", graph_zoom_reset as *const c_void, graph_data as *mut c_void, None);
                connect(zoom_in, "clicked", graph_zoom_in as *const c_void, graph_data as *mut c_void, None);
                connect(zoom_fit, "clicked", graph_zoom_fit as *const c_void, graph_data as *mut c_void, None);
                gtk_drawing_area_set_draw_func(
                    area as *mut GtkDrawingArea,
                    Some(draw_graph),
                    graph_data as *mut c_void,
                    Some(drop_graph_view),
                );
                gtk_scrolled_window_set_policy(sc as *mut GtkScrolledWindow, GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
                gtk_scrolled_window_set_child(sc as *mut GtkScrolledWindow, area);
                gtk_box_append(frame as *mut GtkBox, sc);
                if line.indent > 0 {
                    gtk_widget_set_margin_start(frame, (line.indent as i32) * 22);
                }
                if !search.is_empty() && source.to_lowercase().contains(&search) {
                    gtk_widget_add_css_class(frame, c("search-hit").as_ptr());
                    if found_hits == wanted_hit {
                        gtk_widget_add_css_class(frame, c("search-current").as_ptr());
                        search_widget = frame;
                    }
                    found_hits += 1;
                }
                gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, frame);
                continue;
            }
            if matches!(line.style, Style::Code) {
                let Some(block) = line.code.clone() else {
                    continue;
                };
                let frame = gtk_box_new(GTK_ORIENTATION_VERTICAL, 3);
                gtk_widget_add_css_class(frame, c("md-code-block").as_ptr());
                let head = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
                gtk_widget_add_css_class(head, c("md-code-head").as_ptr());
                let lang = if block.language.is_empty() { "code".to_string() } else { block.language.clone() };
                let lang_label = gtk_label_new(c(&lang).as_ptr());
                if highlight::supported(&block.language) {
                    gtk_widget_set_tooltip_text(lang_label, c("Built-in syntax highlighting enabled").as_ptr());
                }
                gtk_label_set_xalign(lang_label as *mut GtkLabel, 0.0);
                gtk_widget_set_hexpand(lang_label, 1);
                let copy = gtk_button_new_with_label(c("Copy").as_ptr());
                gtk_widget_set_tooltip_text(copy, c("Copy code block").as_ptr());
                let ctx = Box::new(CopyCodeCtx { app: self as *mut App, text: block.text.clone() });
                connect(
                    copy,
                    "clicked",
                    on_copy_code as *const c_void,
                    Box::into_raw(ctx) as *mut c_void,
                    Some(drop_copy_code_ctx),
                );
                gtk_box_append(head as *mut GtkBox, lang_label);
                gtk_box_append(head as *mut GtkBox, copy);
                gtk_box_append(frame as *mut GtkBox, head);

                let sc = gtk_scrolled_window_new();
                let code_lines = block.text.lines().count().max(1) as i32;
                // GtkScrolledWindow does not reliably grow to a GtkLabel's natural height
                // on every GTK4/theme combination. Size short code blocks explicitly so
                // documentation examples are visible without an inner vertical scrollbar.
                // Long blocks stay bounded and scroll vertically.
                const CODE_LINE_HEIGHT: i32 = 21;
                const CODE_VERTICAL_PADDING: i32 = 18;
                const CODE_MAX_HEIGHT: i32 = 640;
                let code_height = (code_lines * CODE_LINE_HEIGHT + CODE_VERTICAL_PADDING).min(CODE_MAX_HEIGHT);
                let long_code = code_lines * CODE_LINE_HEIGHT + CODE_VERTICAL_PADDING > CODE_MAX_HEIGHT;
                gtk_widget_set_size_request(sc, -1, code_height);
                gtk_scrolled_window_set_policy(
                    sc as *mut GtkScrolledWindow,
                    GTK_POLICY_AUTOMATIC,
                    if long_code { GTK_POLICY_AUTOMATIC } else { GTK_POLICY_NEVER },
                );
                let label = gtk_label_new(ptr::null());
                gtk_label_set_xalign(label as *mut GtkLabel, 0.0);
                gtk_label_set_wrap(label as *mut GtkLabel, 0);
                gtk_label_set_selectable(label as *mut GtkLabel, 1);
                gtk_widget_add_css_class(label, c("md-code").as_ptr());
                let highlighted = highlight::render(&block.language, &block.text);
                gtk_label_set_markup(label as *mut GtkLabel, c(&format!("<tt>{}</tt>", highlighted)).as_ptr());
                gtk_scrolled_window_set_child(sc as *mut GtkScrolledWindow, label);
                gtk_box_append(frame as *mut GtkBox, sc);
                if line.indent > 0 {
                    gtk_widget_set_margin_start(frame, (line.indent as i32) * 22);
                }
                if !search.is_empty() && block.text.to_lowercase().contains(&search) {
                    gtk_widget_add_css_class(frame, c("search-hit").as_ptr());
                    if found_hits == wanted_hit {
                        gtk_widget_add_css_class(frame, c("search-current").as_ptr());
                        search_widget = frame;
                    }
                    found_hits += 1;
                }
                gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, frame);
                continue;
            }
            let label = gtk_label_new(ptr::null());
            gtk_label_set_xalign(label as *mut GtkLabel, 0.0);
            gtk_label_set_wrap(label as *mut GtkLabel, 1);
            gtk_label_set_selectable(label as *mut GtkLabel, 1);
            let css = match line.style {
                Style::H1 => "md-h1",
                Style::H2 => "md-h2",
                Style::H3 => "md-h3",
                Style::H4 => "md-h4",
                Style::H5 => "md-h5",
                Style::H6 => "md-h6",
                Style::Bullet => "md-bullet",
                Style::Ordered(_) => "md-bullet",
                Style::Task(_) => "md-task",
                Style::Quote => "md-quote",
                Style::Code => "md-code",
                Style::TableRow => "md-table-row",
                Style::TableSep => "md-table-sep",
                Style::Math => "md-math",
                Style::Diagram => "md-code",
                Style::Footnote => "md-footnote",
                Style::Text => "md-text",
                Style::Rule | Style::Image => unreachable!(),
            };
            gtk_widget_add_css_class(label, c(css).as_ptr());
            let markup = match line.style {
                Style::Bullet => format!("•  {}", line.markup),
                Style::Ordered(n) => format!("{}.  {}", n, line.markup),
                Style::Task(done) => format!("{}  {}", if done { "☑" } else { "☐" }, line.markup),
                Style::Quote => format!("<i>›  {}</i>", line.markup),
                Style::Code => format!("<tt>{}</tt>", line.markup),
                Style::TableSep => String::new(),
                Style::Math => {
                    format!("<span font_family=\"serif\" size=\"large\">{}</span>", line.markup)
                }
                Style::Footnote => line.markup,
                _ => line.markup,
            };
            gtk_label_set_markup(label as *mut GtkLabel, c(&markup).as_ptr());
            if let Some(tip) = &line.tooltip {
                gtk_widget_set_tooltip_text(label, c(tip).as_ptr());
            }
            if !search.is_empty() && strip_pango_markup(&markup).to_lowercase().contains(&search) {
                gtk_widget_add_css_class(label, c("search-hit").as_ptr());
                if found_hits == wanted_hit {
                    gtk_widget_add_css_class(label, c("search-current").as_ptr());
                    gtk_widget_set_focusable(label, 1);
                    search_widget = label;
                }
                found_hits += 1;
            }
            if line.indent > 0 {
                gtk_widget_set_margin_start(label, (line.indent as i32) * 22);
            }
            connect(label, "activate-link", on_inline_link as *const c_void, self as *mut App as *mut c_void, None);
            let matches_primary = target_anchor.as_deref() == line.anchor.as_deref() && line.anchor.is_some();
            let matches_extra = target_anchor.as_ref().map(|a| line.extra_anchors.iter().any(|x| x == a)).unwrap_or(false);
            if matches_primary || matches_extra {
                gtk_widget_set_focusable(label, 1);
                anchor_widget = label;
            }
            gtk_box_append(self.tabs[idx].doc_box as *mut GtkBox, label);
        }

        self.tabs[idx].search_hits = found_hits;
        if found_hits > 0 && self.tabs[idx].search_index >= found_hits {
            self.tabs[idx].search_index = found_hits - 1;
        }
        self.render_tree(idx);
        let focus_widget = if !search_widget.is_null() { search_widget } else { anchor_widget };
        if !focus_widget.is_null() {
            // The idle callback can run after another render has detached this widget.
            // Keep the GObject alive until the callback consumes the reference.
            g_object_ref(focus_widget as *mut c_void);
            if g_idle_add(focus_anchor_idle, focus_widget as *mut c_void) == 0 {
                g_object_unref(focus_widget as *mut c_void);
            }
        }
        if self.stats {
            eprintln!(
                "[stats:render] tab={} blocks={} queued_images={} active_images={} elapsed_ms={:.3}",
                idx + 1,
                lines.len(),
                self.queued_images.len(),
                self.pending_images.len(),
                render_started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    pub(crate) unsafe fn render_tree(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        loop {
            let child = gtk_widget_get_first_child(self.tabs[idx].tree_box);
            if child.is_null() {
                break;
            }
            gtk_box_remove(self.tabs[idx].tree_box as *mut GtkBox, child);
        }

        let bookmarks_heading =
            gtk_button_new_with_label(c(if self.tree_bookmarks_open { "▾  Bookmarks" } else { "▸  Bookmarks" }).as_ptr());
        gtk_widget_add_css_class(bookmarks_heading, c("tree-heading-button").as_ptr());
        let ctx = Box::new(TreeToggleCtx { app: self as *mut App, section: TreeSection::Bookmarks });
        connect(
            bookmarks_heading,
            "clicked",
            on_tree_toggle as *const c_void,
            Box::into_raw(ctx) as *mut c_void,
            Some(drop_tree_toggle_ctx),
        );
        gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, bookmarks_heading);
        if self.tree_bookmarks_open && self.bookmarks.is_empty() {
            let empty = gtk_label_new(c("Ctrl+D adds the current page").as_ptr());
            gtk_label_set_xalign(empty as *mut GtkLabel, 0.0);
            gtk_widget_add_css_class(empty, c("tree-empty").as_ptr());
            gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, empty);
        } else if self.tree_bookmarks_open {
            for item in self.bookmarks.clone().into_iter().take(100) {
                let label = if item.title.is_empty() { &item.url } else { &item.title };
                let btn = gtk_button_new_with_label(c(&format!("★  {}", short(label, 32))).as_ptr());
                gtk_widget_add_css_class(btn, c("tree-link").as_ptr());
                gtk_widget_set_tooltip_text(btn, c(&item.url).as_ptr());
                let ctx = Box::new(LinkCtx { app: self as *mut App, target: item.url });
                connect(btn, "clicked", on_link_clicked as *const c_void, Box::into_raw(ctx) as *mut c_void, Some(drop_link_ctx));
                gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, btn);
            }
        }

        let contents_heading =
            gtk_button_new_with_label(c(if self.tree_contents_open { "▾  Contents" } else { "▸  Contents" }).as_ptr());
        gtk_widget_add_css_class(contents_heading, c("tree-heading-button").as_ptr());
        gtk_widget_set_margin_top(contents_heading, 12);
        let ctx = Box::new(TreeToggleCtx { app: self as *mut App, section: TreeSection::Contents });
        connect(
            contents_heading,
            "clicked",
            on_tree_toggle as *const c_void,
            Box::into_raw(ctx) as *mut c_void,
            Some(drop_tree_toggle_ctx),
        );
        gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, contents_heading);

        let toc_lines = self.tabs[idx].doc.lines.clone();
        let mut toc_count = 0usize;
        if self.tree_contents_open {
            for line in toc_lines {
                let level = match line.style {
                    Style::H1 => 0,
                    Style::H2 => 1,
                    Style::H3 => 2,
                    Style::H4 => 3,
                    Style::H5 => 4,
                    Style::H6 => 5,
                    _ => continue,
                };
                let Some(anchor) = line.anchor else {
                    continue;
                };
                let plain = strip_pango_markup(&line.markup);
                let btn = gtk_button_new_with_label(c(&short(&plain.replace('¶', ""), 34)).as_ptr());
                gtk_widget_add_css_class(btn, c("tree-link").as_ptr());
                gtk_widget_set_margin_start(btn, level * 14);
                let target = format!("#{}", md::percent_encode_fragment(&anchor));
                let ctx = Box::new(LinkCtx { app: self as *mut App, target });
                connect(btn, "clicked", on_link_clicked as *const c_void, Box::into_raw(ctx) as *mut c_void, Some(drop_link_ctx));
                gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, btn);
                toc_count += 1;
                if toc_count >= 200 {
                    break;
                }
            }
        }
        if self.tree_contents_open && toc_count == 0 {
            let empty = gtk_label_new(c("No headings").as_ptr());
            gtk_label_set_xalign(empty as *mut GtkLabel, 0.0);
            gtk_widget_add_css_class(empty, c("tree-empty").as_ptr());
            gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, empty);
        }

        let heading =
            gtk_button_new_with_label(c(if self.tree_links_open { "▾  Markdown links" } else { "▸  Markdown links" }).as_ptr());
        gtk_widget_add_css_class(heading, c("tree-heading-button").as_ptr());
        gtk_widget_set_margin_top(heading, 12);
        let ctx = Box::new(TreeToggleCtx { app: self as *mut App, section: TreeSection::Links });
        connect(
            heading,
            "clicked",
            on_tree_toggle as *const c_void,
            Box::into_raw(ctx) as *mut c_void,
            Some(drop_tree_toggle_ctx),
        );
        gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, heading);
        let links = self.tabs[idx].doc.links.clone();
        if self.tree_links_open {
            for link in links.into_iter().take(200) {
                let btn = gtk_button_new_with_label(c(&short(&link.label, 38)).as_ptr());
                gtk_widget_add_css_class(btn, c("tree-link").as_ptr());
                let ctx = Box::new(LinkCtx { app: self as *mut App, target: link.target });
                connect(btn, "clicked", on_link_clicked as *const c_void, Box::into_raw(ctx) as *mut c_void, Some(drop_link_ctx));
                gtk_box_append(self.tabs[idx].tree_box as *mut GtkBox, btn);
            }
        }
    }
}
