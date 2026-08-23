use super::*;

impl App {
    pub(crate) unsafe fn show_bookmark_manager(&mut self) {
        if self.bookmark_window.is_null() {
            let w = gtk_window_new();
            gtk_window_set_title(w as *mut GtkWindow, c("Bookmarks - SilkMark").as_ptr());
            gtk_window_set_default_size(w as *mut GtkWindow, 720, 480);
            let outer = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
            gtk_widget_set_margin_start(outer, 12);
            gtk_widget_set_margin_end(outer, 12);
            gtk_widget_set_margin_top(outer, 12);
            gtk_widget_set_margin_bottom(outer, 12);
            let head = gtk_label_new(c("Bookmarks: edit title, reorder, or delete").as_ptr());
            gtk_label_set_xalign(head as *mut GtkLabel, 0.0);
            gtk_box_append(outer as *mut GtkBox, head);
            let scroll = gtk_scrolled_window_new();
            gtk_widget_set_hexpand(scroll, 1);
            gtk_widget_set_vexpand(scroll, 1);
            let box_ = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
            gtk_scrolled_window_set_child(scroll as *mut GtkScrolledWindow, box_);
            gtk_box_append(outer as *mut GtkBox, scroll);
            gtk_window_set_child(w as *mut GtkWindow, outer);
            self.bookmark_window = w;
            self.bookmark_box = box_;
            connect(w, "close-request", on_bookmark_manager_close as *const c_void, self as *mut App as *mut c_void, None);
        }
        self.render_bookmark_manager();
        gtk_window_present(self.bookmark_window as *mut GtkWindow);
    }
    pub(crate) unsafe fn render_bookmark_manager(&mut self) {
        if self.bookmark_box.is_null() {
            return;
        }
        loop {
            let child = gtk_widget_get_first_child(self.bookmark_box);
            if child.is_null() {
                break;
            }
            gtk_box_remove(self.bookmark_box as *mut GtkBox, child);
        }
        if self.bookmarks.is_empty() {
            let l = gtk_label_new(c("No bookmarks yet. Ctrl+D adds the current page.").as_ptr());
            gtk_label_set_xalign(l as *mut GtkLabel, 0.0);
            gtk_box_append(self.bookmark_box as *mut GtkBox, l);
            return;
        }
        for item in self.bookmarks.clone() {
            let row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
            let entry = gtk_entry_new();
            set_entry(entry, &item.title);
            gtk_widget_set_hexpand(entry, 1);
            gtk_widget_set_tooltip_text(entry, c(&item.url).as_ptr());
            let up = gtk_button_new_with_label(c("↑").as_ptr());
            let down = gtk_button_new_with_label(c("↓").as_ptr());
            let del = gtk_button_new_with_label(c("Delete").as_ptr());
            let open = gtk_button_new_with_label(c("Open").as_ptr());
            for w in [entry, open, up, down, del] {
                gtk_box_append(row as *mut GtkBox, w);
            }
            let ctx = Box::new(BookmarkEditCtx { app: self as *mut App, url: item.url.clone() });
            connect(
                entry,
                "activate",
                on_bookmark_rename as *const c_void,
                Box::into_raw(ctx) as *mut c_void,
                Some(drop_bookmark_edit_ctx),
            );
            for (w, action) in [
                (open, BookmarkAction::Open),
                (up, BookmarkAction::Up),
                (down, BookmarkAction::Down),
                (del, BookmarkAction::Delete),
            ] {
                let ctx = Box::new(BookmarkActionCtx { app: self as *mut App, url: item.url.clone(), action });
                connect(
                    w,
                    "clicked",
                    on_bookmark_action as *const c_void,
                    Box::into_raw(ctx) as *mut c_void,
                    Some(drop_bookmark_action_ctx),
                );
            }
            gtk_box_append(self.bookmark_box as *mut GtkBox, row);
        }
    }
    pub(crate) unsafe fn bookmark_rename(&mut self, url: &str, title: String) {
        if let Some(b) = self.bookmarks.iter_mut().find(|b| b.url == url) {
            b.title = title;
            let _ = bookmarks::save(&self.bookmarks);
        }
        for i in 0..self.tabs.len() {
            self.render_tree(i);
        }
        self.refresh_status();
    }
    pub(crate) unsafe fn bookmark_action(&mut self, url: &str, action: BookmarkAction) {
        let Some(i) = self.bookmarks.iter().position(|b| b.url == url) else {
            return;
        };
        match action {
            BookmarkAction::Open => {
                let u = self.bookmarks[i].url.clone();
                let tab = self.active;
                self.navigate(tab, u, true, true);
            }
            BookmarkAction::Delete => {
                self.bookmarks.remove(i);
            }
            BookmarkAction::Up => {
                if i > 0 {
                    self.bookmarks.swap(i, i - 1);
                }
            }
            BookmarkAction::Down => {
                if i + 1 < self.bookmarks.len() {
                    self.bookmarks.swap(i, i + 1);
                }
            }
        }
        let _ = bookmarks::save(&self.bookmarks);
        for j in 0..self.tabs.len() {
            self.render_tree(j);
        }
        self.render_bookmark_manager();
        self.refresh_status();
    }
    pub(crate) unsafe fn toggle_bookmark(&mut self) {
        if self.active >= self.tabs.len() {
            return;
        }
        let url = self.tabs[self.active].url.clone();
        if !(url.starts_with("https://") || url.starts_with("http://") || url.starts_with("file://")) {
            self.tabs[self.active].status = "Open a Markdown page before bookmarking".into();
            self.refresh_status();
            return;
        }
        if let Some(pos) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(pos);
            self.tabs[self.active].status = "Bookmark removed".into();
        } else {
            let title = self.tabs[self.active].title.clone();
            self.bookmarks.push(Bookmark { title, url });
            self.tabs[self.active].status = "Bookmark added".into();
        }
        match bookmarks::save(&self.bookmarks) {
            Ok(()) => {}
            Err(e) => self.tabs[self.active].status = format!("Bookmark save failed: {e}"),
        }
        for i in 0..self.tabs.len() {
            self.render_tree(i);
        }
        self.refresh_status();
    }
}
