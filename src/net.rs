use crate::storage;
use std::collections::hash_map::DefaultHasher;
use std::ffi::{CStr, CString, c_char, c_long, c_void};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static DISK_CACHE: AtomicBool = AtomicBool::new(false);
static OFFLINE: AtomicBool = AtomicBool::new(false);
static ALLOW_HTTP: AtomicBool = AtomicBool::new(false);
static MAX_DOCUMENT_SIZE: AtomicUsize = AtomicUsize::new(4 * 1024 * 1024);
static MAX_IMAGE_SIZE: AtomicUsize = AtomicUsize::new(12 * 1024 * 1024);
static MAX_REDIRECTS: AtomicUsize = AtomicUsize::new(8);
static CONNECT_TIMEOUT_MS: AtomicUsize = AtomicUsize::new(10_000);
static TOTAL_TIMEOUT_MS: AtomicUsize = AtomicUsize::new(30_000);
static ALLOWED_HOSTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn set_verbose(value: bool) {
    VERBOSE.store(value, Ordering::Relaxed);
}
pub fn set_disk_cache(value: bool) {
    DISK_CACHE.store(value, Ordering::Relaxed);
}
pub fn set_offline(value: bool) {
    OFFLINE.store(value, Ordering::Relaxed);
    if value {
        DISK_CACHE.store(true, Ordering::Relaxed);
    }
}
pub fn set_allow_http(value: bool) {
    ALLOW_HTTP.store(value, Ordering::Relaxed);
}
fn verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}
fn disk_cache() -> bool {
    DISK_CACHE.load(Ordering::Relaxed)
}
fn offline() -> bool {
    OFFLINE.load(Ordering::Relaxed)
}
fn allow_http() -> bool {
    ALLOW_HTTP.load(Ordering::Relaxed)
}
fn is_network_url(url: &str) -> bool {
    url.starts_with("https://") || (allow_http() && url.starts_with("http://"))
}
pub fn set_limits(document_mib: usize, image_mib: usize, redirects: usize, connect_timeout_s: usize, total_timeout_s: usize) {
    MAX_DOCUMENT_SIZE.store(document_mib.max(1).saturating_mul(1024 * 1024), Ordering::Relaxed);
    MAX_IMAGE_SIZE.store(image_mib.max(1).saturating_mul(1024 * 1024), Ordering::Relaxed);
    MAX_REDIRECTS.store(redirects.min(64), Ordering::Relaxed);
    CONNECT_TIMEOUT_MS.store(connect_timeout_s.max(1).saturating_mul(1000), Ordering::Relaxed);
    TOTAL_TIMEOUT_MS.store(total_timeout_s.max(1).saturating_mul(1000), Ordering::Relaxed);
}
pub fn set_allowed_hosts(hosts: Vec<String>) {
    let v = ALLOWED_HOSTS.get_or_init(|| Mutex::new(Vec::new()));
    *v.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
        hosts.into_iter().map(|h| h.trim().trim_end_matches('.').to_ascii_lowercase()).filter(|h| !h.is_empty()).collect();
}
fn host_of_network_url(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(|c| c == '/' || c == '?' || c == '#').next()?;
    if authority.contains('@') {
        return None;
    }
    let host = if authority.starts_with('[') {
        authority.split(']').next()?.trim_start_matches('[').to_string()
    } else {
        authority.split(':').next()?.to_string()
    };
    if host.is_empty() { None } else { Some(host.trim_end_matches('.').to_ascii_lowercase()) }
}
fn host_allowed(url: &str) -> bool {
    let Some(host) = host_of_network_url(url) else {
        return false;
    };
    let Some(lock) = ALLOWED_HOSTS.get() else {
        return true;
    };
    let list = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if list.is_empty() {
        return true;
    }
    list.iter().any(|rule| {
        if let Some(suffix) = rule.strip_prefix('.') {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == *rule
        }
    })
}
fn configured_document_limit() -> usize {
    MAX_DOCUMENT_SIZE.load(Ordering::Relaxed)
}
fn configured_image_limit() -> usize {
    MAX_IMAGE_SIZE.load(Ordering::Relaxed)
}
const MAX_DISK_CACHE_SIZE: u64 = 64 * 1024 * 1024;
const MAX_CACHE_META_SIZE: usize = 64 * 1024;
static CURL_INIT: OnceLock<Result<(), String>> = OnceLock::new();
const CURL_GLOBAL_ALL: c_long = 3;
const CURLOPT_WRITEDATA: u32 = 10001;
const CURLOPT_URL: u32 = 10002;
const CURLOPT_WRITEFUNCTION: u32 = 20011;
const CURLOPT_USERAGENT: u32 = 10018;
const CURLOPT_FOLLOWLOCATION: u32 = 52;
const CURLOPT_FAILONERROR: u32 = 45;
const CURLOPT_MAXREDIRS: u32 = 68;
const CURLOPT_TIMEOUT_MS: u32 = 155;
const CURLOPT_CONNECTTIMEOUT_MS: u32 = 156;
const CURLOPT_PROTOCOLS_STR: u32 = 10318;
const CURLOPT_REDIR_PROTOCOLS_STR: u32 = 10319;
const CURLOPT_HTTPHEADER: u32 = 10023;
const CURLOPT_HEADERFUNCTION: u32 = 20079;
const CURLOPT_HEADERDATA: u32 = 10029;
const CURLINFO_EFFECTIVE_URL: u32 = 0x100001;
const CURLINFO_RESPONSE_CODE: u32 = 0x200002;

// libcurl URL API. Using the same URL parser as the HTTP client avoids a second,
// subtly different implementation for ../, /root, query strings and fragments.
const CURLUPART_URL: i32 = 0;
const CURLU_DISALLOW_USER: u32 = 1 << 5;
const CURLU_URLENCODE: u32 = 1 << 7;
const CURLU_ALLOW_SPACE: u32 = 1 << 11;

type Curl = c_void;
type CurlU = c_void;
type CurlSlist = c_void;

#[link(name = "curl")]
unsafe extern "C" {
    fn curl_global_init(flags: c_long) -> i32;
    fn curl_easy_init() -> *mut Curl;
    fn curl_easy_cleanup(curl: *mut Curl);
    fn curl_easy_perform(curl: *mut Curl) -> i32;
    fn curl_easy_setopt(curl: *mut Curl, option: u32, ...) -> i32;
    fn curl_easy_getinfo(curl: *mut Curl, info: u32, ...) -> i32;
    fn curl_easy_strerror(code: i32) -> *const c_char;
    fn curl_slist_append(list: *mut CurlSlist, value: *const c_char) -> *mut CurlSlist;
    fn curl_slist_free_all(list: *mut CurlSlist);

    fn curl_url() -> *mut CurlU;
    fn curl_url_cleanup(handle: *mut CurlU);
    fn curl_url_set(handle: *mut CurlU, part: i32, content: *const c_char, flags: u32) -> u32;
    fn curl_url_get(handle: *mut CurlU, part: i32, content: *mut *mut c_char, flags: u32) -> u32;
    fn curl_free(p: *mut c_void);
}

fn ensure_curl_initialized() -> Result<(), String> {
    CURL_INIT
        .get_or_init(|| {
            let rc = unsafe { curl_global_init(CURL_GLOBAL_ALL) };
            if rc == 0 { Ok(()) } else { Err(format!("curl_global_init failed: {rc}")) }
        })
        .clone()
}

#[derive(Default, Clone)]
struct CacheMeta {
    final_url: String,
    etag: String,
    last_modified: String,
}

fn cache_dir() -> Option<PathBuf> {
    if let Some(x) = std::env::var_os("XDG_CACHE_HOME") {
        Some(PathBuf::from(x).join("silkmark"))
    } else {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/silkmark"))
    }
}
fn cache_key(url: &str, kind: &str) -> String {
    let mut h = DefaultHasher::new();
    kind.hash(&mut h);
    url.hash(&mut h);
    format!("{:016x}", h.finish())
}
fn cache_paths(url: &str, kind: &str) -> Option<(PathBuf, PathBuf)> {
    let d = cache_dir()?;
    let k = cache_key(url, kind);
    Some((d.join(format!("{k}.bin")), d.join(format!("{k}.meta"))))
}
pub fn clear_disk_cache() -> Result<(), String> {
    if let Some(d) = cache_dir() {
        if d.exists() {
            fs::remove_dir_all(&d).map_err(|e| format!("Cannot clear {}: {e}", d.display()))?;
        }
    }
    Ok(())
}
fn prune_cache() {
    let Some(d) = cache_dir() else { return };
    let Ok(rd) = fs::read_dir(&d) else { return };
    let mut files: Vec<(std::time::SystemTime, u64, PathBuf)> = Vec::new();
    let mut total = 0u64;
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("bin") {
            continue;
        }
        if let Ok(m) = e.metadata() {
            let sz = m.len();
            total = total.saturating_add(sz);
            files.push((m.modified().unwrap_or(std::time::UNIX_EPOCH), sz, p));
        }
    }
    if total <= MAX_DISK_CACHE_SIZE {
        return;
    }
    files.sort_by_key(|x| x.0);
    for (_, sz, p) in files {
        if total <= MAX_DISK_CACHE_SIZE {
            break;
        }
        let _ = fs::remove_file(&p);
        let _ = fs::remove_file(p.with_extension("meta"));
        total = total.saturating_sub(sz);
    }
}
fn esc_meta(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}
fn unesc_meta(s: &str) -> String {
    let mut o = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => o.push('\n'),
                Some('t') => o.push('\t'),
                Some(x) => o.push(x),
                None => o.push('\\'),
            }
        } else {
            o.push(c)
        }
    }
    o
}
fn load_cache(url: &str, kind: &str, limit: usize) -> Option<(CacheMeta, Vec<u8>)> {
    let (b, m) = cache_paths(url, kind)?;
    let bytes = fs::read(b).ok()?;
    if bytes.len() > limit {
        return None;
    };
    let text = storage::read_text_limited(&m, MAX_CACHE_META_SIZE).ok()?;
    let mut meta = CacheMeta::default();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("final_url\t") {
            meta.final_url = unesc_meta(v)
        } else if let Some(v) = line.strip_prefix("etag\t") {
            meta.etag = unesc_meta(v)
        } else if let Some(v) = line.strip_prefix("last_modified\t") {
            meta.last_modified = unesc_meta(v)
        }
    }
    if meta.final_url.is_empty() {
        meta.final_url = url.to_string()
    }
    Some((meta, bytes))
}
fn save_cache(url: &str, kind: &str, meta: &CacheMeta, bytes: &[u8]) {
    let Some((body_path, meta_path)) = cache_paths(url, kind) else { return };
    let text = format!(
        "final_url\t{}\netag\t{}\nlast_modified\t{}\n",
        esc_meta(&meta.final_url),
        esc_meta(&meta.etag),
        esc_meta(&meta.last_modified)
    );
    if storage::atomic_write(&body_path, bytes).is_err() {
        return;
    }
    if storage::atomic_write(&meta_path, text.as_bytes()).is_err() {
        let _ = fs::remove_file(&body_path);
        return;
    }
    prune_cache();
}

struct HeaderSink {
    etag: String,
    last_modified: String,
}

unsafe extern "C" fn header_cb(data: *mut c_char, size: usize, nmemb: usize, userdata: *mut c_void) -> usize {
    let Some(n) = size.checked_mul(nmemb) else { return 0 };
    if n == 0 {
        return 0;
    }
    if data.is_null() || userdata.is_null() {
        return 0;
    }
    let sink = &mut *(userdata as *mut HeaderSink);
    let bytes = std::slice::from_raw_parts(data as *const u8, n);
    let header = String::from_utf8_lossy(bytes);
    if header.starts_with("HTTP/") {
        sink.etag.clear();
        sink.last_modified.clear();
    } else if let Some((name, value)) = header.split_once(':') {
        if name.eq_ignore_ascii_case("etag") {
            sink.etag = value.trim().to_string();
        } else if name.eq_ignore_ascii_case("last-modified") {
            sink.last_modified = value.trim().to_string();
        }
    }
    n
}

struct Sink {
    bytes: Vec<u8>,
    too_large: bool,
    limit: usize,
}

unsafe extern "C" fn write_cb(data: *mut c_char, size: usize, nmemb: usize, userdata: *mut c_void) -> usize {
    let Some(n) = size.checked_mul(nmemb) else { return 0 };
    if n == 0 {
        return 0;
    }
    if data.is_null() || userdata.is_null() {
        return 0;
    }
    let sink = &mut *(userdata as *mut Sink);
    if sink.bytes.len().saturating_add(n) > sink.limit {
        sink.too_large = true;
        return 0;
    }
    let part = std::slice::from_raw_parts(data as *const u8, n);
    sink.bytes.extend_from_slice(part);
    n
}

/// RFC-style URL joining using libcurl itself. Relative paths such as
/// `img/a.jpg`, `../img/a.jpg`, `/img/a.jpg`, `?v=2` and `#section` are handled.
/// Spaces/non-ASCII bytes in the target are URL-encoded by libcurl.
pub fn resolve_network_url(base: &str, target: &str, keep_fragment: bool) -> Option<String> {
    if base.contains('\0') || target.contains('\0') || !is_network_url(base) {
        return None;
    }
    let base = CString::new(base).ok()?;
    let target = CString::new(target.trim()).ok()?;
    ensure_curl_initialized().ok()?;
    unsafe {
        let h = curl_url();
        if h.is_null() {
            return None;
        }
        let base_flags = CURLU_DISALLOW_USER | CURLU_ALLOW_SPACE;
        if curl_url_set(h, CURLUPART_URL, base.as_ptr(), base_flags) != 0 {
            curl_url_cleanup(h);
            return None;
        }
        let target_flags = CURLU_DISALLOW_USER | CURLU_ALLOW_SPACE | CURLU_URLENCODE;
        if curl_url_set(h, CURLUPART_URL, target.as_ptr(), target_flags) != 0 {
            curl_url_cleanup(h);
            return None;
        }
        let mut out: *mut c_char = ptr::null_mut();
        if curl_url_get(h, CURLUPART_URL, &mut out, 0) != 0 || out.is_null() {
            curl_url_cleanup(h);
            return None;
        }
        let mut url = CStr::from_ptr(out).to_string_lossy().into_owned();
        curl_free(out as *mut c_void);
        curl_url_cleanup(h);
        if !is_network_url(&url) {
            return None;
        }
        if !keep_fragment {
            if let Some(i) = url.find('#') {
                url.truncate(i);
            }
        }
        Some(url)
    }
}

fn split_fragment(s: &str) -> (&str, Option<&str>) {
    match s.find('#') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    }
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn pct_decode(s: &str) -> Option<String> {
    let bs = s.as_bytes();
    let mut out = Vec::with_capacity(bs.len());
    let mut i = 0;
    while i < bs.len() {
        if bs[i] == b'%' && i + 2 < bs.len() {
            let hi = hex_value(bs[i + 1])?;
            let lo = hex_value(bs[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bs[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn pct_encode_path(s: &str) -> String {
    let mut out = String::new();
    for &b in s.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'~' | b':') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let (doc, _) = split_fragment(url);
    let raw = doc.strip_prefix("file://")?;
    // Local-only browser semantics: no remote file://host/path authorities.
    if !raw.starts_with('/') {
        return None;
    }
    Some(PathBuf::from(pct_decode(raw)?))
}

fn path_to_file_url(path: &Path, fragment: Option<&str>) -> Option<String> {
    let abs = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().ok()?.join(path) };
    let normalized = fs::canonicalize(&abs).unwrap_or(abs);
    let text = normalized.to_string_lossy();
    let mut u = format!("file://{}", pct_encode_path(&text));
    if let Some(f) = fragment {
        if !f.is_empty() {
            u.push('#');
            u.push_str(f);
        }
    }
    Some(u)
}

/// Normalize a user-entered location. HTTPS URLs are retained; HTTP URLs are
/// accepted only when --allow-http is enabled; file:// URLs and
/// ordinary filesystem paths become absolute file:// URLs. A trailing #fragment
/// is retained so local section deep-links work exactly like web documents.
pub fn normalize_location(input: &str) -> Option<String> {
    let t = input.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("https://") || (allow_http() && t.starts_with("http://")) {
        return if host_allowed(t) { Some(t.to_string()) } else { None };
    }
    if t.starts_with("file://") {
        let (_, frag) = split_fragment(t);
        return path_to_file_url(&file_url_to_path(t)?, frag);
    }
    if t.starts_with("data:") || t.starts_with("javascript:") || t.starts_with("http:") || t.starts_with("ftp:") {
        return None;
    }
    if t.contains("://") {
        return None;
    }
    let (path, frag) = split_fragment(t);
    path_to_file_url(Path::new(path), frag)
}

/// Resolve Markdown links and image references for network and local files.
/// HTTP is accepted only when --allow-http is enabled.
pub fn resolve_resource(base: &str, target: &str, keep_fragment: bool) -> Option<String> {
    let target = target.trim();
    if target.is_empty() {
        return None;
    }
    if target.starts_with("data:") || target.starts_with("javascript:") || target.starts_with("ftp:") {
        return None;
    }
    if target.starts_with("http://") && !allow_http() {
        return None;
    }
    if is_network_url(base) {
        return resolve_network_url(base, target, keep_fragment);
    }
    if !base.starts_with("file://") {
        return None;
    }
    if target.starts_with("https://") || (allow_http() && target.starts_with("http://")) {
        if !host_allowed(target) {
            return None;
        }
        let mut u = target.to_string();
        if !keep_fragment {
            if let Some(i) = u.find('#') {
                u.truncate(i);
            }
        }
        return Some(u);
    }
    if target.contains("://") && !target.starts_with("file://") {
        return None;
    }
    if target.starts_with("file://") {
        let mut u = normalize_location(target)?;
        if !keep_fragment {
            if let Some(i) = u.find('#') {
                u.truncate(i);
            }
        }
        return Some(u);
    }
    let (base_doc, _) = split_fragment(base);
    let base_path = file_url_to_path(base_doc)?;
    let (tpath, frag) = split_fragment(target);
    if tpath.is_empty() {
        let mut u = path_to_file_url(&base_path, if keep_fragment { frag } else { None })?;
        if !keep_fragment {
            if let Some(i) = u.find('#') {
                u.truncate(i);
            }
        }
        return Some(u);
    }
    let path = if Path::new(tpath).is_absolute() {
        PathBuf::from(tpath)
    } else {
        base_path.parent().unwrap_or(Path::new("/")).join(tpath)
    };
    path_to_file_url(&path, if keep_fragment { frag } else { None })
}

fn read_local(url: &str, limit: usize, kind: &str) -> Result<(String, Vec<u8>), String> {
    let path = file_url_to_path(url).ok_or_else(|| format!("Invalid local {kind} URL"))?;
    if verbose() {
        eprintln!("[open:{kind}] {}", url);
    }
    let meta = fs::metadata(&path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    if !meta.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }
    if meta.len() as usize > limit {
        return Err(format!("{kind} exceeds {} MiB limit", limit / 1024 / 1024));
    }
    let bytes = fs::read(&path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    if bytes.len() > limit {
        return Err(format!("{kind} exceeds {} MiB limit", limit / 1024 / 1024));
    }
    Ok((path_to_file_url(&path, None).unwrap_or_else(|| url.to_string()), bytes))
}

fn perform(url: &str, limit: usize, kind: &str) -> Result<(String, Vec<u8>), String> {
    if verbose() {
        eprintln!("[open:{kind}] {url}");
    }
    if !is_network_url(url) {
        return Err(format!("Unsupported network {kind} URL"));
    }
    if !host_allowed(url) {
        return Err(format!(
            "Host is not allowed by --allow-host: {}",
            host_of_network_url(url).unwrap_or_else(|| "<invalid>".into())
        ));
    }
    let cached = if disk_cache() { load_cache(url, kind, limit) } else { None };
    if offline() {
        if let Some((meta, bytes)) = cached {
            if verbose() {
                eprintln!("[cache:disk-hit:{kind}] {url}");
            }
            return Ok((meta.final_url, bytes));
        }
        if verbose() {
            eprintln!("[cache:disk-miss:{kind}] {url}");
        }
        return Err(format!("Offline mode: {kind} not found in disk cache"));
    }
    if verbose() && disk_cache() {
        eprintln!("[cache:disk-{}:{kind}] {url}", if cached.is_some() { "revalidate" } else { "miss" });
    }
    let c_url = CString::new(url).map_err(|_| "URL contains NUL".to_string())?;
    let ua =
        CString::new(format!("SilkMark/{}", env!("CARGO_PKG_VERSION"))).map_err(|_| "User-Agent contains NUL".to_string())?;
    let protocols =
        CString::new(if allow_http() { "http,https" } else { "https" }).map_err(|_| "Protocol list contains NUL".to_string())?;
    let mut sink = Sink { bytes: Vec::with_capacity(64 * 1024), too_large: false, limit };
    let mut hs = HeaderSink { etag: String::new(), last_modified: String::new() };
    ensure_curl_initialized()?;
    unsafe {
        let h = curl_easy_init();
        if h.is_null() {
            return Err("curl_easy_init failed".into());
        }
        let set = |opt, value: *const c_char| curl_easy_setopt(h, opt, value);
        if set(CURLOPT_URL, c_url.as_ptr()) != 0
            || set(CURLOPT_USERAGENT, ua.as_ptr()) != 0
            || set(CURLOPT_PROTOCOLS_STR, protocols.as_ptr()) != 0
            || set(CURLOPT_REDIR_PROTOCOLS_STR, protocols.as_ptr()) != 0
            || curl_easy_setopt(h, CURLOPT_FOLLOWLOCATION, 1 as c_long) != 0
            || curl_easy_setopt(h, CURLOPT_FAILONERROR, 1 as c_long) != 0
            || curl_easy_setopt(h, CURLOPT_MAXREDIRS, MAX_REDIRECTS.load(Ordering::Relaxed) as c_long) != 0
            || curl_easy_setopt(h, CURLOPT_CONNECTTIMEOUT_MS, CONNECT_TIMEOUT_MS.load(Ordering::Relaxed) as c_long) != 0
            || curl_easy_setopt(h, CURLOPT_TIMEOUT_MS, TOTAL_TIMEOUT_MS.load(Ordering::Relaxed) as c_long) != 0
            || curl_easy_setopt(h, CURLOPT_WRITEFUNCTION, write_cb as *const c_void) != 0
            || curl_easy_setopt(h, CURLOPT_WRITEDATA, &mut sink as *mut Sink as *mut c_void) != 0
            || curl_easy_setopt(h, CURLOPT_HEADERFUNCTION, header_cb as *const c_void) != 0
            || curl_easy_setopt(h, CURLOPT_HEADERDATA, &mut hs as *mut HeaderSink as *mut c_void) != 0
        {
            curl_easy_cleanup(h);
            return Err("curl option setup failed".into());
        }
        let mut headers: *mut CurlSlist = ptr::null_mut();
        let mut owned: Vec<CString> = Vec::new();
        if let Some((meta, _)) = &cached {
            if !meta.etag.is_empty() {
                if let Ok(value) = CString::new(format!("If-None-Match: {}", meta.etag)) {
                    owned.push(value);
                    if let Some(value) = owned.last() {
                        headers = curl_slist_append(headers, value.as_ptr());
                    }
                }
            }
            if !meta.last_modified.is_empty() {
                if let Ok(value) = CString::new(format!("If-Modified-Since: {}", meta.last_modified)) {
                    owned.push(value);
                    if let Some(value) = owned.last() {
                        headers = curl_slist_append(headers, value.as_ptr());
                    }
                }
            }
            if !headers.is_null() {
                let _ = curl_easy_setopt(h, CURLOPT_HTTPHEADER, headers);
            }
        }
        let rc = curl_easy_perform(h);
        let mut code: c_long = 0;
        let _ = curl_easy_getinfo(h, CURLINFO_RESPONSE_CODE, &mut code);
        let mut effective: *mut c_char = ptr::null_mut();
        let _ = curl_easy_getinfo(h, CURLINFO_EFFECTIVE_URL, &mut effective);
        let final_url =
            if effective.is_null() { url.to_string() } else { CStr::from_ptr(effective).to_string_lossy().into_owned() };
        if !headers.is_null() {
            curl_slist_free_all(headers)
        }
        curl_easy_cleanup(h);
        if code == 304 {
            if let Some((meta, bytes)) = cached {
                if verbose() {
                    eprintln!("[cache:304:{kind}] {url}");
                }
                return Ok((meta.final_url, bytes));
            }
        }
        if rc != 0 {
            let msg = if sink.too_large {
                format!("{kind} exceeds {} MiB limit", limit / 1024 / 1024)
            } else {
                match rc {
                    28 => format!("Network timeout while loading {kind}"),
                    35 => "TLS handshake failed".to_string(),
                    51 => "TLS certificate does not match the requested host".to_string(),
                    58 => "TLS client certificate problem".to_string(),
                    60 => "TLS certificate verification failed".to_string(),
                    _ => {
                        let p = curl_easy_strerror(rc);
                        if p.is_null() { format!("curl error {rc}") } else { CStr::from_ptr(p).to_string_lossy().into_owned() }
                    }
                }
            };
            return Err(msg);
        }
        if !is_network_url(&final_url) {
            return Err(format!("{kind} redirect used a disallowed protocol"));
        }
        if !host_allowed(&final_url) {
            return Err(format!(
                "Redirected host is not allowed by --allow-host: {}",
                host_of_network_url(&final_url).unwrap_or_else(|| "<invalid>".into())
            ));
        }
        if verbose() && final_url != url {
            eprintln!("[redirect:{kind}] {url} -> {final_url}");
        }
        if disk_cache() {
            let meta = CacheMeta { final_url: final_url.clone(), etag: hs.etag, last_modified: hs.last_modified };
            save_cache(url, kind, &meta, &sink.bytes);
            if verbose() {
                eprintln!("[cache:disk-store:{kind}] {url} ({} bytes)", sink.bytes.len());
            }
        }
        Ok((final_url, sink.bytes))
    }
}

pub fn fetch(url: &str) -> Result<(String, String), String> {
    let (final_url, bytes) = if is_network_url(url) {
        perform(url, configured_document_limit(), "document")?
    } else if url.starts_with("file://") {
        read_local(url, configured_document_limit(), "document")?
    } else {
        return Err("Only allowed HTTP(S) URLs and local files are supported".into());
    };
    let text = String::from_utf8(bytes).map_err(|_| "Document is not UTF-8 Markdown".to_string())?;
    Ok((final_url, text))
}

pub fn fetch_image(url: &str) -> Result<Vec<u8>, String> {
    let (final_url, bytes) = if is_network_url(url) {
        perform(url, configured_image_limit(), "image")?
    } else if url.starts_with("file://") {
        read_local(url, configured_image_limit(), "image")?
    } else {
        return Err("Only allowed HTTP(S) URLs and local image files are supported".into());
    };
    if verbose() {
        eprintln!("[data:image] {final_url} ({} bytes)", bytes.len());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_asset_url() {
        assert_eq!(
            resolve_network_url("https://example.org/docs/readme.md", "images/a.jpg", false).as_deref(),
            Some("https://example.org/docs/images/a.jpg")
        );
    }

    #[test]
    fn parent_relative_asset_url() {
        assert_eq!(
            resolve_network_url("https://example.org/docs/ch1/readme.md", "../images/a.jpg?v=2", false).as_deref(),
            Some("https://example.org/docs/images/a.jpg?v=2")
        );
    }

    #[test]
    fn root_relative_asset_url() {
        assert_eq!(
            resolve_network_url("https://example.org/docs/readme.md", "/assets/a.jpg", false).as_deref(),
            Some("https://example.org/assets/a.jpg")
        );
    }

    #[test]
    fn relative_markdown_link_with_fragment() {
        assert_eq!(
            resolve_network_url("https://example.org/docs/ch1/a.md", "../api/ref.md#Request format", true).as_deref(),
            Some("https://example.org/docs/api/ref.md#Request%20format")
        );
    }

    #[test]
    fn fragment_only_relative_link() {
        assert_eq!(
            resolve_network_url("https://example.org/docs/a.md#old", "#Local section", true).as_deref(),
            Some("https://example.org/docs/a.md#Local%20section")
        );
    }

    #[test]
    fn local_relative_markdown_link() {
        let base = path_to_file_url(Path::new("/tmp/silkmark/docs/ch1/a.md"), None).unwrap();
        assert_eq!(
            resolve_resource(&base, "../api/ref.md#Request format", true).as_deref(),
            Some("file:///tmp/silkmark/docs/api/ref.md#Request format")
        );
    }

    #[test]
    fn unsafe_schemes_are_rejected() {
        assert!(normalize_location("javascript:alert(1)").is_none());
        assert!(normalize_location("data:text/plain,hello").is_none());
        let base = "https://example.org/docs/a.md";
        assert!(resolve_resource(base, "http://example.org/a.md", true).is_none());
        assert!(resolve_resource(base, "ftp://example.org/a.md", true).is_none());
    }

    #[test]
    fn local_relative_image() {
        let base = path_to_file_url(Path::new("/tmp/silkmark/docs/a.md"), None).unwrap();
        assert_eq!(resolve_resource(&base, "images/pic.jpg", false).as_deref(), Some("file:///tmp/silkmark/docs/images/pic.jpg"));
    }
}
