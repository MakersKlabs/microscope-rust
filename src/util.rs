//! Small helpers: timestamps, paths and default folders.

use std::path::{Path, PathBuf};

/// Local timestamp used in file names, e.g. `20260101_123456`.
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
}

fn home_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_default()
}

/// Default folder for pictures, falling back to the home directory.
pub fn image_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.picture_dir().map(Path::to_path_buf))
        .unwrap_or_else(home_dir)
}

/// Default folder for videos, falling back to the home directory.
pub fn video_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.video_dir().map(Path::to_path_buf))
        .unwrap_or_else(home_dir)
}

/// Ensure the path has one of the accepted extensions, otherwise append `default`.
pub fn ensure_extension(path: PathBuf, accepted: &[&str], default: &str) -> PathBuf {
    let ok = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            accepted
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false);
    if ok {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(default);
        PathBuf::from(name)
    }
}
