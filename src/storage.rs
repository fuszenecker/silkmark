use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path(path: &Path) -> io::Result<PathBuf> {
    let parent =
        path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target path has no parent directory"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target filename is not valid UTF-8"))?;
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(".{name}.tmp.{}.{}", std::process::id(), seq)))
}

/// Atomically replaces `path` with `bytes` by writing and syncing a unique
/// temporary file in the same directory and then renaming it into place.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent =
        path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target path has no parent directory"))?;
    fs::create_dir_all(parent)?;

    let (tmp, mut file) = loop {
        let tmp = temp_path(path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&tmp) {
            Ok(file) => break (tmp, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };

    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp, path)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// Reads a UTF-8 text file with a hard byte limit. The limit is enforced while
/// reading, so a malformed or locally replaced state/cache file cannot force an
/// unbounded allocation before validation.
pub fn read_text_limited(path: &Path, max_bytes: usize) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(max_bytes.min(64 * 1024));
    file.take((max_bytes as u64).saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "text file exceeds size limit"));
    }
    String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "text file is not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_text_limited_rejects_oversized_input() {
        let base = std::env::temp_dir().join(format!(
            "silkmark-limited-read-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&base).expect("create test directory");
        let path = base.join("large.txt");
        fs::write(&path, b"12345").expect("seed file");
        assert!(read_text_limited(&path, 4).is_err());
        assert_eq!(read_text_limited(&path, 5).expect("bounded read"), "12345");
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn atomic_write_replaces_existing_file() {
        let base = std::env::temp_dir().join(format!(
            "silkmark-atomic-write-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&base).expect("create test directory");
        let path = base.join("state.tsv");
        fs::write(&path, b"old").expect("seed file");
        atomic_write(&path, b"new\ncontent").expect("atomic replacement");
        assert_eq!(fs::read(&path).expect("read replacement"), b"new\ncontent");
        let _ = fs::remove_dir_all(base);
    }
}
