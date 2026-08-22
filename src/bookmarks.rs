use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::storage;

const MAX_BOOKMARKS: usize = 4096;
const MAX_BOOKMARK_LINE_LEN: usize = 64 * 1024;
const MAX_BOOKMARK_FILE_SIZE: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

fn data_file() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(dir).join("silkmark/bookmarks.tsv"));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/silkmark/bookmarks.tsv"))
}

fn legacy_data_file() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(dir).join("md-browser-arachne/bookmarks.tsv"));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/md-browser-arachne/bookmarks.tsv"))
}

fn supported_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("file://")
}

fn parse_text(text: &str) -> Vec<Bookmark> {
    text.lines()
        .take(MAX_BOOKMARKS)
        .filter(|line| line.len() <= MAX_BOOKMARK_LINE_LEN)
        .filter_map(|line| {
            let (title, url) = line.split_once('\t')?;
            if !supported_url(url) {
                return None;
            }
            Some(Bookmark { title: title.to_string(), url: url.to_string() })
        })
        .collect()
}

pub fn load() -> Vec<Bookmark> {
    let Some(path) = data_file() else {
        return Vec::new();
    };
    let text = storage::read_text_limited(&path, MAX_BOOKMARK_FILE_SIZE).or_else(|_| {
        let legacy = legacy_data_file().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no legacy path"))?;
        storage::read_text_limited(&legacy, MAX_BOOKMARK_FILE_SIZE)
    });
    let Ok(text) = text else {
        return Vec::new();
    };
    parse_text(&text)
}

fn clean_title(s: &str) -> String {
    s.chars().map(|ch| if matches!(ch, '\t' | '\n' | '\r') { ' ' } else { ch }).collect()
}

fn clean_url(s: &str) -> String {
    s.chars().filter(|ch| !matches!(ch, '\t' | '\n' | '\r')).collect()
}

pub fn save(items: &[Bookmark]) -> io::Result<()> {
    let path = data_file().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME/XDG_DATA_HOME is not set"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut body = String::new();
    for item in items.iter().take(MAX_BOOKMARKS) {
        let url = clean_url(&item.url);
        if !supported_url(&url) {
            continue;
        }
        body.push_str(&clean_title(&item.title));
        body.push('\t');
        body.push_str(&url);
        body.push('\n');
    }
    storage::atomic_write(&path, body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_https_and_local_file_bookmarks() {
        let items =
            parse_text("Remote\thttps://example.org/readme.md\nLocal\tfile:///tmp/readme.md\nBad\thttp://example.org/x.md\n");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.url.starts_with("file://")));
    }

    #[test]
    fn ignores_unsupported_bookmark_schemes() {
        let items = parse_text("JS\tjavascript:alert(1)\nData\tdata:text/plain,hello\n");
        assert!(items.is_empty());
    }
}
