use super::*;

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
                if node == self.tabs[idx].doc_view {
                    inside_document = true;
                    break;
                }
                node = gtk_widget_get_parent(node);
            }
            if inside_document {
                gtk_window_set_focus(self.window as *mut GtkWindow, ptr::null_mut());
            }
        }

        self.show_document(idx);
        self.render_tree(idx);
        if self.stats {
            eprintln!(
                "[stats:render] tab={} blocks={} queued_images={} active_images={} elapsed_ms={:.3}",
                idx + 1,
                self.tabs[idx].doc.index.blocks.len(),
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
