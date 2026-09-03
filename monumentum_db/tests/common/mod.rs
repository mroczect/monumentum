use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempPath {
    path: PathBuf,
    is_dir: bool,
}

impl TempPath {
    pub fn new_file(prefix: &str) -> Self {
        Self::new(prefix, false)
    }

    pub fn new_dir(prefix: &str) -> Self {
        Self::new(prefix, true)
    }

    fn new(prefix: &str, is_dir: bool) -> Self {
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos,
            count
        ));
        if is_dir {
            std::fs::create_dir_all(&path).expect("failed to create temp dir");
        }
        Self { path, is_dir }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if self.is_dir {
            let _ = std::fs::remove_dir_all(&self.path);
        } else {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
